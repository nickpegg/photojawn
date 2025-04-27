import logging
import sys
from argparse import ArgumentParser, Namespace
from pathlib import Path

from rich.logging import RichHandler

from photojawn.config import DEFAULT_CONFIG_PATH, Config
from photojawn.generate import generate

logger = logging.getLogger("photojawn.cli")


def main() -> None:
    args = parse_args()
    setup_logging(args.logging)

    # Load config from file if it exists
    if hasattr(args, "album_path"):
        conf_path = Path(args.album_path) / Path(args.config)
        if conf_path.exists():
            logger.debug(f"Reading config from {conf_path}")
            config = Config.from_yaml(conf_path.read_bytes())
        elif args.action != "init":
            logger.error(
                f"No config file found at {conf_path}. If this is a new photo directory, "
                "please run `photojawn init` in there first."
            )
            return

    # Call the subcommand function
    match args.action:
        case "init":
            cmd_init(args)
        case "generate":
            if args.quick:
                config.quick = args.quick
            cmd_generate(args, config)
        case "clean":
            cmd_clean(args, config)


def parse_args() -> Namespace:
    parser = ArgumentParser()
    parser.add_argument(
        "--config",
        "-c",
        default=DEFAULT_CONFIG_PATH,
        help="Path to photojawn.config.json for the album",
    )
    parser.add_argument(
        "--logging",
        default="warning",
        choices=[level.lower() for level in logging.getLevelNamesMapping().keys()],
        help="Log level",
    )

    subcommands = parser.add_subparsers(title="subcommands")

    init_cmd = subcommands.add_parser(
        "init",
        help="Initialize an photo directory",
    )
    init_cmd.set_defaults(action="init")
    init_cmd.add_argument(
        "album_path",
        nargs="?",
        default=".",
        help="Path to the main photos directory",
    )

    # Generate subcommand
    generate_cmd = subcommands.add_parser(
        "generate",
        help="Generate the HTML photo album",
    )
    generate_cmd.set_defaults(action="generate")
    generate_cmd.add_argument(
        "--quick",
        action="store_true",
        help="Quick mode - don't regenerate thumbnails",
    )
    generate_cmd.add_argument(
        "album_path",
        nargs="?",
        default=".",
        help="Path to the main photos directory",
    )

    # Clean subcommand
    clean_cmd = subcommands.add_parser(
        "clean",
        help="Remove all generated content from the photo album directory",
    )
    clean_cmd.set_defaults(action="clean")
    clean_cmd.add_argument(
        "album_path",
        nargs="?",
        default=".",
        help="Path to the main photos directory",
    )

    args = parser.parse_args()
    if not hasattr(args, "action"):
        parser.print_help()
        sys.exit(0)

    return args


########################################
# Command functions
def cmd_init(args: Namespace) -> None:
    """
    Generate a basic config and template files
    """
    album_path = Path(args.album_path)
    config_path = album_path / args.config
    if config_path.exists():
        logger.warning(
            f"Looks like {album_path} is already set up. If you want to start over and "
            f"overwrite any of your customizations, remove {config_path}"
        )
        return

    skel_dir = Path(__file__).parent / "skel"
    logger.debug(f"Skeleton dir: {skel_dir}")

    skel_files = []
    for parent_path, dirnames, filenames in skel_dir.walk():
        for filename in filenames:
            skel_file_path = parent_path / filename
            rel_path = skel_file_path.relative_to(skel_dir)
            album_file_path = album_path / rel_path

            skel_files.append(album_file_path)

            album_file_path.parent.mkdir(exist_ok=True)
            album_file_path.write_bytes(skel_file_path.read_bytes())
            logger.debug(f"Created skeleton file {album_file_path}")

    print("Some basic files have been created for your album. Edit them as you need:")
    for p in skel_files:
        print(f" - {p}")


def cmd_generate(args: Namespace, config: Config) -> None:
    logger.debug(f"Generating in {args.album_path}")
    generate(config, Path(args.album_path))


def cmd_clean(args: Namespace, config: Config) -> None:
    """
    Clean the photo album by all files that photojawn generated
    """
    pass


########################################
# CLI Util functions
def setup_logging(level_str: str) -> None:
    levels = logging.getLevelNamesMapping()
    level = levels[level_str.upper()]
    logging.basicConfig(
        level=level,
        format="[%(name)s] %(message)s",
        handlers=[RichHandler(rich_tracebacks=True)],
    )
    # Override PIL logging because debug is really noisy
    if level <= logging.DEBUG:
        logging.getLogger("PIL").setLevel(logging.INFO)


if __name__ == "__main__":
    main()
