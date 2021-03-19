defmodule CrayonsTest do
  use ExUnit.Case
  doctest Crayons

  test "greets the world" do
    assert Crayons.hello() == :world
  end
end
