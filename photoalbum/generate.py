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
    photos = find_photos(album_path)
    generate_thumbnails(config, photos)
    generate_html(config, album_path)


def find_photos(path: Path) -> list[Path]:
    """ """
    photos = []
    for parent_path, dirnames, filenames in path.walk():
        if parent_path.name == "slides":
            continue

        for filename in filenames:
            file_path = parent_path / filename
            if is_photo(file_path):
                photos.append(file_path)

    return photos


def is_photo(photo_path: Path) -> bool:
    """
    Returns True if PIL thinks the file is a photo
    """
    try:
        Image.open(photo_path)
        return True
    except UnidentifiedImageError:
        return False


def generate_thumbnails(config: Config, photos: list[Path]) -> None:
    """
    Find all of the images and generate thumbnails and on-screen versions
    """
    for photo_path in track(photos, description="Making thumbnails..."):
        orig_img = Image.open(photo_path)

        slides_path = photo_path.parent / "slides"
        slides_path.mkdir(exist_ok=True)

        thumb_img = orig_img.copy()
        thumb_img.thumbnail(config.thumbnail_size)
        thumb_filename = photo_path.stem + ".thumb" + photo_path.suffix
        thumb_img.save(slides_path / thumb_filename)
        logger.info(f"Generated thumbnail size {photo_path} -> {thumb_filename}")

        screen_img = orig_img.copy()
        screen_img.thumbnail(config.view_size)
        screen_filename = photo_path.stem + ".screen" + photo_path.suffix
        screen_img.save(slides_path / screen_filename)
        logger.info(f"Generated screen size {photo_path} -> {screen_filename}")


def generate_html(config: Config, path: Path) -> None:
    """
    Recursively generate HTML files for this directory and all children
    """
