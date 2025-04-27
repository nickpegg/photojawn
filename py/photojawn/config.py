import logging
from dataclasses import dataclass

import yaml

DEFAULT_CONFIG_PATH = "photojawn.conf.yml"

logger = logging.getLogger(__name__)


@dataclass
class Config:
    # Size of thumbnails when looking at a folder page
    thumbnail_size: tuple[int, int] = (256, 256)

    # Size of the image when looking at the standalone image page
    view_size: tuple[int, int] = (1920, 1080)

    # Directory inside the photo directory to output the site to
    output_dir: str = "site"

    # Quick mode:
    # - Don't regenerate thumbnails if they already exist
    quick: bool = False

    @classmethod
    def from_yaml(cls, contents: bytes) -> "Config":
        conf = cls()
        data = yaml.safe_load(contents)
        if data is None:
            return conf

        for key, val in data.items():
            match key:
                case "output_dir":
                    conf.output_dir = val
                case "thumnail_size":
                    conf.thumbnail_size = tuple(val)
                case "view_size":
                    conf.view_size = tuple(val)
        return conf
