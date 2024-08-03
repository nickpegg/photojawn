import logging
from pathlib import Path

from PIL import Image, UnidentifiedImageError
from rich.progress import track

from photoalbum.config import Config

logger = logging.getLogger(__name__)


def generate(config: Config, album_path: Path) -> None:
    """
    Main generation function
    """
    generate_thumbnails(config, album_path)
    # generate_html(config, album_path)


def generate_thumbnails(config: Config, path: Path) -> None:
    """
    Find all of the images and generate thumbnails and on-screen versions
    """
    images: list[tuple[Path, str]] = []
    for parent_path, dirnames, filenames in path.walk():
        for filename in filenames:
            try:
                Image.open(parent_path / filename)
            except UnidentifiedImageError:
                continue  # Not an image file

            # If we modify dirnames in-place, walk() will skip anything we remove
            if "slides" in dirnames:
                dirnames.remove("slides")

            images.append((parent_path, filename))

    for parent_path, filename in track(images, description="Making thumbnails..."):
        file_path = parent_path / filename
        orig_img = Image.open(parent_path / filename)

        slides_path = parent_path / "slides"
        slides_path.mkdir(exist_ok=True)

        thumb_img = orig_img.copy()
        thumb_img.thumbnail(config.thumbnail_size)
        thumb_filename = file_path.stem + ".thumb" + file_path.suffix
        thumb_img.save(slides_path / thumb_filename)
        logger.info(
            f"Generated thumbnail size {parent_path / filename} -> {thumb_filename}"
        )

        screen_img = orig_img.copy()
        screen_img.thumbnail(config.view_size)
        screen_filename = file_path.stem + ".screen" + file_path.suffix
        screen_img.save(slides_path / screen_filename)
        logger.info(
            f"Generated screen size {parent_path / filename} -> {screen_filename}"
        )


def generate_html(config: Config, path: Path) -> None:
    """
    Recursively generate HTML files for this directory and all children
    """
