defmodule CrayonsTest do
  use ExUnit.Case
  doctest Crayons

  test "leaves unknown text alone" do
    assert {:ok, "Hello, world!"} = "Hello, world!" |> Crayons.color(:unknown, format: :terminal)
    assert {:ok, "Hello, world!"} = "Hello, world!" |> Crayons.color(:unknown, format: :html)
  end

  test "adds HTML even to plaintext" do
    assert {:ok,
            "<pre style=\"background-color:#002b36;\">\n<span style=\"color:#839496;\">Hello, world!</span></pre>\n"} =
             "Hello, world!" |> Crayons.color(nil, format: :html)
  end

  test "can load new definitions" do
    name = "testing"
    assert nil == Crayons.list_themes |> Enum.find(fn theme -> theme == name end)
    assert {:ok, name} ==
             "assets/InspiredGithub.tmTheme"
             |> File.read()
             |> Crayons.add_theme(name)
    assert name == Crayons.list_themes |> Enum.find(fn theme -> theme == name end)
  end
end
