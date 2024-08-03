import logging
from argparse import ArgumentParser, Namespace
from pathlib import Path

from photoalbum.config import DEFAULT_CONFIG_PATH, Config
from photoalbum.generate import generate

logger = logging.getLogger("photoalbum.cli")


def main() -> None:
    args = parse_args()
    setup_logging(args.logging)
    # TODO: load config from file
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
    logger.debug(f"Generating in {args.path}")
    generate(config, Path(args.path))


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

    subcommands = parser.add_subparsers(title="subcommands")

    # Generate subcommand
    generate_cmd = subcommands.add_parser(
        "generate",
        help="Generate the HTML photo album",
    )
    generate_cmd.set_defaults(action="generate")
    generate_cmd.add_argument(
        "path",
        nargs="?",
        default=".",
        help="Path to dir with photos in it",
    )

    # Clean subcommand
    clean_cmd = subcommands.add_parser(
        "clean",
        help="Remove all generated content from the photo album directory",
    )
    clean_cmd.set_defaults(action="clean")

    return parser.parse_args()


def setup_logging(level: str) -> None:
    levels = logging.getLevelNamesMapping()
    # TODO: Set up a better formatter with date/time and stuff
    logging.basicConfig(level=levels[level.upper()])


if __name__ == "__main__":
    main()
