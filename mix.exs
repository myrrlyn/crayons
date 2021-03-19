defmodule Crayons.MixProject do
  use Mix.Project

  def project do
    [
      app: :crayons,
      version: "0.1.0",
      description: description(),
      elixir: "~> 1.11",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      package: package(),
      source_url: "https://github.com/myrrlyn/crayons",
      docs: [main: "Crayons"],
      build_embedded: Mix.env() == :prod,
      start_permanent: Mix.env() == :prod,
      compilers: [:rustler | Mix.compilers()],
      rustler_crates: rustler_crates()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp description do
    """
    Crayons is a binding to the Rust `syntect` project for syntax highlighting.
    """
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, "~> 0.21"},
      {:ex_doc, "~> 0.24"}
      # {:dep_from_hexpm, "~> 0.3.0"},
      # {:dep_from_git, git: "https://github.com/elixir-lang/my_dep.git", tag: "0.1.0"}
    ]
  end

  defp package do
    [
      maintainers: ["myrrlyn"],
      licenses: ["MIT"],
      files: [
        "lib",
        "native",
        "mix.exs",
        "README.md",
        "LICENSE.txt",
        links: %{"GitHub" => "https://github.com/myrrlyn/crayons"}
      ]
    ]
  end

  defp rustler_crates do
    [
      crayons_nif: [
        path: "native/crayons",
        cargo: :system,
        default_features: true,
        features: [],
        mode: if(Mix.env() == :prod, do: :release, else: :debug)
      ]
    ]
  end
end
