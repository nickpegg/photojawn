import logging
from argparse import ArgumentParser, Namespace
from pathlib import Path

from photoalbum.config import DEFAULT_CONFIG_PATH, Config
from photoalbum.generate import generate
from rich.logging import RichHandler

logger = logging.getLogger("photoalbum.cli")


def main() -> None:
    args = parse_args()
    setup_logging(args.logging)

    # Load config from file if it exists
    conf_path = Path(args.album_path) / Path(args.config)
    if conf_path.exists():
        logger.debug(f"Reading config from {conf_path}")
        config = Config.from_yaml(conf_path.read_bytes())
    else:
        logger.warning(f"No config file found at {conf_path}. Using defaults")
        config = Config()

    # Call the subcommand function
    match args.action:
        case "generate":
            cmd_generate(args, config)
        case "clean":
            cmd_clean(args, config)


########################################
# Command functions
def cmd_init(args: Namespace, config: Config) -> None:
    """
    Generate a basic config and template files
    """


def cmd_generate(args: Namespace, config: Config) -> None:
    logger.debug(f"Generating in {args.album_path}")
    generate(config, Path(args.album_path))


def cmd_clean(args: Namespace, config: Config) -> None:
    """
    Clean the photo album by all files that photoalbum generated
    """
    pass


########################################
# CLI Util functions
def parse_args() -> Namespace:
    parser = ArgumentParser()
    parser.add_argument(
        "--config",
        "-c",
        default=DEFAULT_CONFIG_PATH,
        help="Path to photoalbum.config.json for the album",
    )
    parser.add_argument(
        "--logging",
        default="warning",
        choices=[level.lower() for level in logging.getLevelNamesMapping().keys()],
        help="Log level",
    )
    parser.add_argument(
        "--album-path",
        "-p",
        default=".",
        help="Path to the main photos directory",
    )

    subcommands = parser.add_subparsers(title="subcommands")

    # Generate subcommand
    generate_cmd = subcommands.add_parser(
        "generate",
        help="Generate the HTML photo album",
    )
    generate_cmd.set_defaults(action="generate")

    # Clean subcommand
    clean_cmd = subcommands.add_parser(
        "clean",
        help="Remove all generated content from the photo album directory",
    )
    clean_cmd.set_defaults(action="clean")

    return parser.parse_args()


def setup_logging(level_str: str) -> None:
    levels = logging.getLevelNamesMapping()
    level = levels[level_str.upper()]
    logging.basicConfig(
        level=level,
        handlers=[RichHandler(rich_tracebacks=True)],
    )
    # Override PIL logging because debug is really noisy
    if level <= logging.DEBUG:
        logging.getLogger("PIL").setLevel(logging.INFO)



if __name__ == "__main__":
    main()
