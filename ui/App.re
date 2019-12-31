[@bs.val] external document: Js.t({..}) = "document";

let container = document##createElement("main");
let () = document##body##appendChild(container);

ReactDOMRe.render(<div>{React.string("Hello world")}</div>, container);
