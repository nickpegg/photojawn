import logging
from dataclasses import dataclass
from pathlib import Path
from pprint import pformat
from typing import Iterator

from jinja2 import Environment, FileSystemLoader, select_autoescape
from PIL import Image, UnidentifiedImageError
from rich.progress import Progress, track

from photoalbum.config import Config

logger = logging.getLogger(__name__)


@dataclass
class ImageDirectory:
    path: Path
    children: list["ImageDirectory"]
    images: list["ImagePath"]
    is_root: bool = False

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


@dataclass
class ImagePath:
    path: Path

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
    root_dir = find_images(album_path)
    logger.debug(pformat(root_dir))
    generate_thumbnails(config, root_dir)
    generate_html(config, root_dir)


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
            dirpath, ImageDirectory(path=dirpath, children=[], images=[])
        )

        for dirname in sorted(dirnames):
            child_path = dirpath / dirname
            if child_path in image_dirs:
                image_dir.children.append(image_dirs[child_path])

        for filename in sorted(filenames):
            file_path = dirpath / filename
            if is_image(file_path):
                image_dir.images.append(ImagePath(file_path))

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
    for image_path in track(root_dir.image_paths(), description="Making thumbnails..."):
        logger.debug(image_path)
        orig_img = Image.open(image_path.path)

        slides_path = image_path.path.parent / "slides"
        slides_path.mkdir(exist_ok=True)

        thumb_img = orig_img.copy()
        thumb_img.thumbnail(config.thumbnail_size)
        thumb_path = image_path.thumbnail_path()
        thumb_img.save(thumb_path)
        logger.info(f'Generated thumbnail size "{image_path.path}" -> "{thumb_path}"')

        screen_img = orig_img.copy()
        screen_img.thumbnail(config.view_size)
        screen_path = image_path.display_path()
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
            static_path = root_dir.path.relative_to(
                html_path.parent, walk_up=True
            ) / "static"

            logger.debug(f"Rendering {html_path}")
            with html_path.open("w") as f:
                f.write(
                    album_tmpl.render(
                        static_dir=static_path,
                        album_dir=album_dir,
                    )
                )

            for pos, image_path in enumerate(album_dir.images):
                # TODO: If a file with a matching name but .txt or .md, add that as the
                # description for the image
                html_path = image_path.html_path()
                static_path = root_dir.path.relative_to(
                    html_path.parent, walk_up=True
                ) / "static"
                html_path.parent.mkdir(exist_ok=True)

                prev_image = None
                next_image = None
                if pos != 0:
                    prev_image = album_dir.images[pos-1]
                if pos < len(album_dir.images) - 1:
                    next_image = album_dir.images[pos+1]

                logger.debug(f"Rendering {html_path}")
                with html_path.open("w") as f:
                    f.write(
                        photo_tmpl.render(
                            static_dir=static_path,
                            image_path=image_path,
                            prev_image=prev_image,
                            next_image=next_image,
                        )
                    )

                progress.update(task, advance=1)
