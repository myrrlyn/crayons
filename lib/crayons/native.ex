defmodule Crayons.Native do
  @moduledoc """
  Bindings to the Rust crate in `native/crayons`. Not public API.
  """

  use Rustler, otp_app: :crayons, crate: "crayons_nif"

  defmodule NifNotLoaded do
    @moduledoc """
    An exception raised when the crayons NIF is not available.
    """

    defexception message: "Crayons NIF not loaded"
  end

  @doc """
  Calls `crayons_nif::color`.

  See [`Crayons.color`].
  """
  @spec color(
          String.t(),
          atom | String.t(),
          :html | :terminal,
          String.t()
        ) :: {:ok, String.t()} | {:error, atom}
  def color(_text, _lang, _format, _theme) do
    raise NifNotLoaded
  end

  @doc """
  Calls `crayons_nif::add_lang`.

  See [`Crayons.add_lang`].
  """
  @spec add_lang(
          binary,
          String.t() | nil,
          true | false | nil
        ) :: {:ok, String.t()} | {:error, String.t()}
  def add_lang(_content, _name \\ nil, _include_newlines \\ false) do
    raise NifNotLoaded
  end

  @doc """
  Calls `crayons_nif::add_theme`.

  See [`Crayons.add_theme`].
  """
  @spec add_theme(
          binary,
          String.t()
        ) :: {:ok, String.t()} | {:error, String.t()}
  def add_theme(_content, _name) do
    raise NifNotLoaded
  end

  def list_langs(), do: raise(NifNotLoaded)

  def list_themes(), do: raise(NifNotLoaded)
end
