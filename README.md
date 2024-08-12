# photojawn

This is a super-simple photo album static site generator. You feed it a
directory of photos (which can contain directories of photos, etc. etc.) and
it'll generate a basic HTML photo album for you. You can then host the
directory with a webserver of your choice or upload it to an S3 bucket.

It's everything I need and nothing I don't.


## Getting Started

### Installation

Photojawn requires at least Python 3.12. If you don't know what version you
have, you can check by running `python -V`.

1. Head on over to the [releases](https://github.com/nickpegg/photojawn/releases) page
2. If you have Python >=3.12, you can install using the wheel (`.whl` file)
  a. To install, run `pip install photojawn-<version>-py3-none-any.whl`
3. If you don't have Python >=3.12, you can download one of the standalone binaries depending on your OS and architecture, e.g. `photojawn-linux-x86_64`.


### Initialization

Then inside your photo directory, run:
```
photojawn init
```

This will create a config file, some [jinja2](https://jinja.palletsprojects.com/en/latest/templates/)
HTML templates, and a CSS file. Edit them to your heart's content to make your
photo album website purdy.

### Generating the site

To generate the HTML files and various image sizes, inside your photo
directory, run:
```
photojawn generate
```


## Special features

- HTML templates are written using [jinja2](https://jinja.palletsprojects.com/en/latest/templates/)
- If you have a `description.txt` or `description.md` file in a directory with
  images, its contents will be used as the album description. `.md` files will
  be rendered as Markdown.
- If an image file (e.g. `IMG_1234.jpg`) has a corresponding `.txt` or `.md`
  file (e.g. `IMG_1234.md`) then it'll be used as the image's caption. `.md`
  files will be rendered as Markdown.
- If you have an image in a directory called `cover.jpg` (or a symlink
  to another image named that), then it'll be used as the cover image for the
  album. If one doesn't exist, the first image in the directory will be used as
  the cover image.


## y tho

Why create a new photo album doohickey? Why not use one of the untold number of
cloud services or even self-hosted solutions? It boils down to a few things:

1. I want control of my data. I don't want some company using my pictures to
   train their AI models, for example.
2. A lot of the self-hosted solutions (Immich, PhotoPrism, etc.) don't support
   nested albums
3. I love simplicity. I'm following the [Unix philosophy](https://en.wikipedia.org/wiki/Unix_philosophy)
   here: "do one thing, do it well" and make use of composable tools to get the
   job done.

I took heavy inspiration by the photo albums found on https://bayarearides.com
([example](https://bayarearides.com/rides/annadel1/photos/)).
