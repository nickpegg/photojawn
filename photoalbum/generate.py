import logging
from dataclasses import dataclass
from pathlib import Path
from pprint import pformat
from typing import Iterator

from jinja2 import Environment, FileSystemLoader, select_autoescape
from PIL import Image, UnidentifiedImageError
from rich.console import Console
from rich.progress import track

from photoalbum.config import Config

logger = logging.getLogger(__name__)


@dataclass
class ImageDirectory:
    path: Path
    children: list["ImageDirectory"]
    images: list[Path]
    is_root: bool = False

    def walk(self) -> Iterator["ImageDirectory"]:
        yield self
        for child in self.children:
            yield from child.walk()

    def image_paths(self) -> list[Path]:
        """
        Iterate through all images in this dir and children
        """
        images = []
        for image_dir in self.walk():
            images += image_dir.images
        return images


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
                image_dir.images.append(file_path)

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
        orig_img = Image.open(image_path)

        slides_path = image_path.parent / "slides"
        slides_path.mkdir(exist_ok=True)

        thumb_img = orig_img.copy()
        thumb_img.thumbnail(config.thumbnail_size)
        thumb_filename = image_path.stem + ".thumb" + image_path.suffix
        thumb_img.save(slides_path / thumb_filename)
        logger.info(f'Generated thumbnail size "{image_path}" -> "{thumb_filename}"')

        screen_img = orig_img.copy()
        screen_img.thumbnail(config.view_size)
        screen_filename = image_path.stem + ".screen" + image_path.suffix
        screen_img.save(slides_path / screen_filename)
        logger.info(f'Generated screen size "{image_path}" -> "{screen_filename}"')


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

    with Console().status("Rendering HTML..."):
        for album_dir in root_dir.walk():
            html_path = album_dir.path / "index.html"
            logger.debug(f"Rendering {html_path}")
            with html_path.open("w") as f:
                f.write(album_tmpl.render())

            for image_path in album_dir.images:
                # TODO: If a file with a matching name but .txt or .md, add that as the
                # description for the image
                html_path = (
                    image_path.parent / "slides" / image_path.with_suffix(".html").name
                )
                html_path.parent.mkdir(exist_ok=True)
                logger.debug(f"Rendering {html_path}")
                with html_path.open("w") as f:
                    f.write(photo_tmpl.render())
