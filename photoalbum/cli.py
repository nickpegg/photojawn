from argparse import ArgumentParser, Namespace
from pathlib import Path

from photoalbum.config import DEFAULT_CONFIG_PATH, Config
from photoalbum.generate import generate


def main() -> None:
    args = parse_args()
    # TODO: load config from file
    config = Config()

    # Call the subcommand function
    match args.action:
        case "generate":
            cmd_generate(args, config)
        case "clean":
            cmd_clean(args, config)


def parse_args() -> Namespace:
    parser = ArgumentParser()
    parser.add_argument(
        "--config",
        "-c",
        default=DEFAULT_CONFIG_PATH,
        help="Path to photoalbum.config.json for the album",
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


def cmd_init(args: Namespace, config: Config) -> None:
    """
    Generate a basic config and template files
    """


def cmd_generate(args: Namespace, config: Config) -> None:
    generate(config, Path(args.path))


def cmd_clean(args: Namespace, config: Config) -> None:
    """
    Clean the photo album by all files that photoalbum generated
    """
    pass


if __name__ == "__main__":
    main()
