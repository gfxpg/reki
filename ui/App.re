[@bs.val] external document: Js.t({..}) = "document";
let container = document##querySelector("main");

let render = reki => {
  ReactDOMRe.render(<Main />, container);
};

Reki.instantiate(3, 5)
|> Js.Promise.(then_(reki => reki |> render |> resolve));
