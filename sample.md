# CommonMark sample document

## Basic inline formatting

Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam **nonumy
eirmod tempor invidunt** ut labore et *dolore magna aliquyam erat*, sed diam
voluptua. `At vero eos et` accusam et

## Links

Another paragraph has an auto-link (see <https://www.example.com>) inside.

TK: More kinds of links

## Lists

Unordered lists:

* Lorem impsum
    * Nested
    * Inline
    * Text
* dolor sit amet
    * Nested

    * With

      Paragraphs and nested blocks:

      > A quote

      And some text at the end
* consetetur sadipscing elitr

Ordered lists:

1. Lorem impsum
    1. Nested
    2. Inline
    3. Text
2. dolor sit amet
    1. Nested

    2. With

      Paragraphs and nested blocks:

      > A quote

      And some text at the end
3. consetetur sadipscing elitr

## Block level elements

Block quotes

> Lorem ipsum dolor sit amet, *consetetur sadipscing elitr*, sed diam nonumy
> eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam
> voluptua.
>
> Lorem ipsum dolor sit amet, **consetetur sadipscing elitr**, sed diam nonumy
> eirmod tempor invidunt ut `labore et dolore magna` aliquyam erat, sed diam
> voluptua.

Before we continue, have a ruler:

----

Code blocks without syntax highlighting:

```
Some plain
code block
   fooo
```

Or with syntax highlighting, eg, Rust:

```rust
fn main() {
    println!("Hello world")
}
```

Or Haskell:

```haskell
main :: IO ()
main = putStrLn "Hello World"
```

Or Scala:

```scala
object HelloWorld {
  def main(args: Array[String]): Unit = {
    println("Hello, world!")
  }
}
```