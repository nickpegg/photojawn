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
    images = find_images(album_path)
    generate_thumbnails(config, images)
    generate_html(config, album_path)


def find_images(path: Path) -> list[Path]:
    """
    Returns paths of all images in the given path
    """
    images = []
    for parent_path, dirnames, filenames in path.walk():
        if parent_path.name == "slides":
            continue

        for filename in filenames:
            file_path = parent_path / filename
            if is_image(file_path):
                images.append(file_path)

    return images


def is_image(path: Path) -> bool:
    """
    Returns True if PIL thinks the file is an image
    """
    try:
        Image.open(path)
        return True
    except UnidentifiedImageError:
        return False


def generate_thumbnails(config: Config, images: list[Path]) -> None:
    """
    Find all of the images and generate thumbnails and on-screen versions
    """
    for image_path in track(images, description="Making thumbnails..."):
        orig_img = Image.open(image_path)

        slides_path = image_path.parent / "slides"
        slides_path.mkdir(exist_ok=True)

        thumb_img = orig_img.copy()
        thumb_img.thumbnail(config.thumbnail_size)
        thumb_filename = image_path.stem + ".thumb" + image_path.suffix
        thumb_img.save(slides_path / thumb_filename)
        logger.info(f"Generated thumbnail size {image_path} -> {thumb_filename}")

        screen_img = orig_img.copy()
        screen_img.thumbnail(config.view_size)
        screen_filename = image_path.stem + ".screen" + image_path.suffix
        screen_img.save(slides_path / screen_filename)
        logger.info(f"Generated screen size {image_path} -> {screen_filename}")


def generate_html(config: Config, path: Path) -> None:
    """
    Recursively generate HTML files for this directory and all children
    """
