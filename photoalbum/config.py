from dataclasses import dataclass

DEFAULT_CONFIG_PATH = "photoalbum.conf.yml"


@dataclass
class Config:
    # Size of thumbnails when looking at a folder page
    thumbnail_size: tuple[int, int] = (128, 128)

    # Size of the image when looking at the standalone image page
    view_size: tuple[int, int] = (1920, 1080)

    # TODO: to/from file classmethods
