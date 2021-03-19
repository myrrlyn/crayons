# Crayons

This is a wrapper over the Rust [`syntect`] project, and provides syntax
highlighting for Elixir projects.

## Usage

The primary function is `Crayons.color`, which receives some text to highlight,
a language marker, and a coloring theme.

```elixir
text = """
defmodule CrayonsExample do
  @moduledoc false

  def method(), do: nil
end
"""
{:ok, html}  = text |> Crayons.color(:elixir)
{:ok, ansi}  = text |> Crayons.color(:elixir, format: :terminal)
{:ok, lite}  = text |> Crayons.color(:elixir, theme: "Solarized (light)")
```

You can query which languages and themes are available, and you can supply your
own by reading the contents of `.tmLanguage` and `.tmTheme` files into the
library:

```elixir
{:ok, _} = "new_lang.tmLanguage" |> File.read |> Crayons.add_lang()
{:ok, _} = "new_theme.tmTheme" |> File.read |> Crayons.add_theme("new")

langs = Crayons.list_langs()
themes = Crayons.list_themes()
```

## Lock Safety

You should generally use the `.add_` methods during application boot, and
generally not during runtime. While you may add additional data during runtime,
doing so contends with coloring text, and race conditions between adding new
data and coloring text will result in blocking the *system* threads that the
BEAM is using.

I will change this to only block the BEAM task once I figure out how to make
native libraries yield to the scheduler.

## Installation

If [available in Hex](https://hex.pm/docs/publish), the package can be installed
by adding `crayons` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:crayons, "~> 0.1.0"}
  ]
end
```

Documentation can be generated with [ExDoc](https://github.com/elixir-lang/ex_doc)
and published on [HexDocs](https://hexdocs.pm). Once published, the docs can
be found at [https://hexdocs.pm/crayons](https://hexdocs.pm/crayons).

[`syntect`]: https://crates.io/crates/syntect
