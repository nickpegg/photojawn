from pathlib import Path

from rich.progress import track
from PIL import Image, UnidentifiedImageError

from photoalbum.config import Config


def generate(config: Config, path: Path) -> None:
    """
    Main generation function
    """
    skeleton_created = maybe_create_skeleton(config)
    generate_thumbnails(config, path)
    generate_html_dir(config, path)


def maybe_create_skeleton(config: Config) -> bool:
    """
    Create basic config, template, and static files if they don't already exist in the
    repo
    """
    return False


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
                continue    # Not an image file

            # If we modify dirnames in-place, walk() will skip anything we remove
            if 'slides' in dirnames:
                dirnames.remove('slides')

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

        screen_img = orig_img.copy()
        screen_img.thumbnail(config.view_size)
        screen_filename = file_path.stem + ".screen" + file_path.suffix
        screen_img.save(slides_path / screen_filename)


def generate_html_dir(config: Config, path: Path) -> None:
    """
    Recursively generate HTML files for this directory and all children
    """
