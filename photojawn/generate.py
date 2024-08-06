import logging
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator, Optional

from jinja2 import Environment, FileSystemLoader, select_autoescape
from markdown import markdown
from PIL import Image, UnidentifiedImageError
from rich.progress import Progress, track

from photojawn.config import Config

logger = logging.getLogger(__name__)


@dataclass
class ImageDirectory:
    path: Path
    children: list["ImageDirectory"]
    images: list["ImagePath"]
    is_root: bool = False
    description: str = ""

    cover_path: Optional["ImagePath"] = None

    def walk(self) -> Iterator["ImageDirectory"]:
        yield self
        for child in self.children:
            yield from child.walk()

    def image_paths(self) -> list["ImagePath"]:
        """
        Iterate through all images in this dir and children
        """
        images = []
        for image_dir in self.walk():
            images += image_dir.images
        return images

    def cover_image_paths(self) -> list["ImagePath"]:
        images = []
        for image_dir in self.walk():
            if image_dir.cover_path is not None:
                images.append(image_dir.cover_path)
        return images


@dataclass
class ImagePath:
    path: Path
    description: str = ""

    def thumbnail_filename(self) -> str:
        return self.path.stem + ".thumb" + self.path.suffix

    def thumbnail_path(self) -> Path:
        return self.path.parent / "slides" / self.thumbnail_filename()

    def display_filename(self) -> str:
        return self.path.stem + ".screen" + self.path.suffix

    def display_path(self) -> Path:
        return self.path.parent / "slides" / self.display_filename()

    def html_filename(self) -> str:
        return self.path.with_suffix(".html").name

    def html_path(self) -> Path:
        return self.path.parent / "slides" / self.html_filename()


def generate(config: Config, album_path: Path) -> None:
    """
    Main generation function
    """
    # Change the working directory to the album_path so that all paths are relative to
    # it when we find images. We need to do this because all the paths in HTML need to
    # be relative to it and we don't want to have to do a bunch of path gymnastics to
    # re-relative all those paths.
    orig_wd = Path.cwd()
    os.chdir(album_path)

    root_dir = find_images(Path("."))
    generate_thumbnails(config, root_dir)
    generate_html(config, root_dir)

    os.chdir(orig_wd)


def find_images(root_path: Path) -> ImageDirectory:
    """
    Build up an ImageDirectory to track all of the directories and their images.

    A directory with no images nor childern with images will not be tracked, since we
    don't want to render an album page for those.
    """
    # image_dirs keeps track of all directories we find with images in them, so we can
    # attach them as children to parent directories
    image_dirs: dict[Path, ImageDirectory] = {
        root_path: ImageDirectory(path=root_path, children=[], images=[], is_root=True)
    }

    for dirpath, dirnames, filenames in root_path.walk(top_down=False):
        if dirpath.name in {"slides", "_templates", "static"}:
            continue

        image_dir = image_dirs.get(
            dirpath,
            ImageDirectory(
                path=dirpath,
                children=[],
                images=[],
            ),
        )

        for dirname in sorted(dirnames):
            child_path = dirpath / dirname
            if child_path in image_dirs:
                image_dir.children.append(image_dirs[child_path])

        for filename in sorted(filenames):
            file_path = dirpath / filename
            if filename == "description.txt":
                image_dir.description = file_path.read_text()
            elif filename == "description.md":
                image_dir.description = markdown(file_path.read_text())
            elif is_image(file_path):
                ip = ImagePath(file_path)

                # Set a cover image for the album. Use "cover.jpg" if one exists,
                # otherwise use the first image we find.
                if file_path.stem == "cover":
                    image_dir.cover_path = ip
                    # Don't add the cover image to the list of images, we want to handle
                    # that separately
                    continue

                # If there's an associated .txt or .md file, read it in as the image's
                # description
                if file_path.with_suffix(".md").exists():
                    ip.description = markdown(file_path.with_suffix(".md").read_text())
                elif file_path.with_suffix(".txt").exists():
                    ip.description = file_path.with_suffix(".txt").read_text()

                image_dir.images.append(ip)

        if image_dir.cover_path is None:
            if len(image_dir.images) > 0:
                image_dir.cover_path = image_dir.images[0]
            elif len(image_dir.children) > 0:
                cover = image_dir.children[0].cover_path
                logger.debug(f"nested cover path for {image_dir.path.name}: {cover}")
                image_dir.cover_path = cover

        image_dirs[image_dir.path] = image_dir

    return image_dirs[root_path]


def is_image(path: Path) -> bool:
    """
    Returns True if PIL thinks the file is an image
    """
    try:
        Image.open(path)
        return True
    except UnidentifiedImageError:
        return False


def generate_thumbnails(config: Config, root_dir: ImageDirectory) -> None:
    """
    Find all of the images and generate thumbnails and on-screen versions
    """
    # Include cover images here because we want thumbnails for all of them

    all_images = root_dir.image_paths() + root_dir.cover_image_paths()
    for image_path in track(all_images, description="Making thumbnails..."):
        orig_img = Image.open(image_path.path)

        slides_path = image_path.path.parent / "slides"
        slides_path.mkdir(exist_ok=True)

        thumb_path = image_path.thumbnail_path()
        if not thumb_path.exists() or not config.quick:
            thumb_img = orig_img.copy()
            thumb_img.thumbnail(config.thumbnail_size)
            thumb_img.save(thumb_path)
            logger.info(
                f'Generated thumbnail size "{image_path.path}" -> "{thumb_path}"'
            )

        screen_path = image_path.display_path()
        if not screen_path.exists() or not config.quick:
            screen_img = orig_img.copy()
            screen_img.thumbnail(config.view_size)
            screen_img.save(screen_path)
            logger.info(f'Generated screen size "{image_path.path}" -> "{screen_path}"')


def generate_html(config: Config, root_dir: ImageDirectory) -> None:
    """
    Recursively generate HTML files for this directory and all children
    """
    jinja_env = Environment(
        loader=FileSystemLoader(root_dir.path / "_templates"),
        autoescape=select_autoescape(),
    )

    album_tmpl = jinja_env.get_template("album.html")
    photo_tmpl = jinja_env.get_template("photo.html")

    with Progress() as progress:
        task = progress.add_task("Rendering HTML...", total=len(root_dir.image_paths()))

        for album_dir in root_dir.walk():
            html_path = album_dir.path / "index.html"
            root_path = root_dir.path.relative_to(html_path.parent, walk_up=True)

            # TODO build breadcrumbs here, (href, name)
            breadcrumbs = []
            if not album_dir.is_root:
                crumb_pos = album_dir.path.parent
                while crumb_pos != root_dir.path:
                    breadcrumbs.append(
                        (
                            str(crumb_pos.relative_to(album_dir.path, walk_up=True)),
                            crumb_pos.name,
                        )
                    )
                    crumb_pos = crumb_pos.parent
            breadcrumbs.reverse()

            logger.debug(f"Rendering {html_path}")
            with html_path.open("w") as f:
                f.write(
                    album_tmpl.render(
                        root_path=root_path,
                        album_dir=album_dir,
                        breadcrumbs=breadcrumbs,
                    )
                )

            for pos, image_path in enumerate(album_dir.images):
                # TODO: If a file with a matching name but .txt or .md, add that as the
                # description for the image
                if image_path.path.stem == "cover":
                    continue

                html_path = image_path.html_path()
                root_path = root_dir.path.relative_to(html_path.parent, walk_up=True)
                html_path.parent.mkdir(exist_ok=True)

                prev_image = None
                next_image = None
                if pos != 0:
                    prev_image = album_dir.images[pos - 1]
                if pos < len(album_dir.images) - 1:
                    next_image = album_dir.images[pos + 1]

                logger.debug(f"Rendering {html_path}")
                with html_path.open("w") as f:
                    f.write(
                        photo_tmpl.render(
                            root_path=root_path,
                            image_path=image_path,
                            prev_image=prev_image,
                            next_image=next_image,
                        )
                    )

                progress.update(task, advance=1)
