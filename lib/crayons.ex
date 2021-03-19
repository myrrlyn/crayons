defmodule Crayons do
  @moduledoc """
  `Crayons` provides a binding to the Rust [`syntect`] project for syntax
  highlighting.

  In order to build this library, you will need to install the Rust project
  (<https://rustup.rs>).

  [`syntect`]: https://crates.io/crates/syntect
  """

  @type format :: :html | :terminal
  @type opt :: {:format, format} | {:theme, String.t()}
  @type error :: {:error, atom}

  @doc """
  Colors some text according to a language specifier and a theme.

  ## Arguments

  - `text`: A string of some text to format according to a language and theme.
  - `lang`: An atom or string that names a language. This must be one of the
    languages known to [`syntect`], or a language that you have added to the
    library with [`Crayons.add_lang`]. If it is `nil` or the empty string, then
    the plaintext formatter is chosen.
  - `opts`:
    - `format:` must be one of `:html` or `:terminal`
    - `theme:` must be a string name that is known to [`syntect`] as a theme,
      either by default or added with [`Crayons.add_theme`].

  [`syntect`]: https://crates.io/crates/syntect
  """
  @spec color(
          String.t(),
          atom | String.t() | nil,
          keyword
        ) :: {:ok, String.t()} | {:error, atom, String.t() | nil}
  def color(text, lang \\ nil, opts \\ [])

  # Empty-string and nil language markers use plaintext
  def color(text, "", opts), do: color(text, nil, opts)
  def color(text, nil, opts), do: color(text, "txt", opts)

  # Forward to the NIF-call
  def color(text, lang, opts) do
    theme = opts |> Keyword.get(:theme, "Solarized (dark)")
    format = opts |> Keyword.get(:format, :html)

    fn -> Crayons.Native.color(text, lang, format, theme) end |> offload
  end

  @doc """
  Adds a new language to the library's understanding.

  This requires that the language definition be loaded into the BEAM before use;
  it will not read from the filesystem itself. Use it as:

  ```elixir
  path |> File.read!() |> Crayons.add_lang()
  ```

  ## Arguments

  - `file`: The *contents* of a `.tmLanguage` file. If this is a result tuple
    (`{:ok | :error, contents | error}`), then it will do nothing in the error
    case and forward in the success case.
  - `name`: A fallback name to give to the language, if `file` does not contain
    one.
  - `opts`:
    - `with_newlines`: Whether or not the new grammar expects source text for
      colorization to contain newlines.
  """
  @spec add_lang(binary | {:ok, binary} | {:error, File.posix()}, String.t() | nil, keyword) ::
          {:ok, String.t()} | {:error, String.t()}
  def add_lang(file, name \\ nil, opts \\ [])

  def add_lang({:ok, file}, name, opts), do: add_lang(file, name, opts)
  def add_lang({:error, err}, _name, _opts), do: {:error, err}

  def add_lang(file, name, opts) do
    with_newlines = opts |> Keyword.get(:with_newlines, false)
    fn -> Crayons.Native.add_lang(file, name, with_newlines) end |> offload
  end

  @doc """
  Adds a new theme to the library's understanding.

  This requires that the theme definition be loaded into the BEAM before use; it
  will not read from the filesystem itself. Use it as:

  ```elixir
  path |> File.read!() |> Crayons.add_theme()
  ```

  ## Arguments

  - `file`: The *contents* of a `.tmTheme` file. If this is a result tuple
    (`{:ok | :error, contents | error}`), then it will do nothing in the error
    case and forward in the success case.
  - `name`: The name of the theme.
  """
  @spec add_theme(binary | {:ok, binary} | {:error, File.posix()}, String.t()) ::
          {:ok, String.t()} | {:error, String.t()}
  def add_theme(file, name)

  def add_theme({:ok, file}, name), do: add_theme(file, name)
  def add_theme({:error, err}, _name), do: {:error, err}

  def add_theme(file, name), do: fn -> Crayons.Native.add_theme(file, name) end |> offload

  @spec list_langs() :: [String.t()]
  def list_langs(), do: Crayons.Native.list_langs()

  @spec list_themes() :: [String.t()]
  def list_themes(), do: Crayons.Native.list_themes()

  defp offload(func), do: func |> Task.async() |> Task.await()
end
