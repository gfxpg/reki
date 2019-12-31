[@bs.val] external document: Js.t({..}) = "document";

let container = document##createElement("main");
let () = document##body##appendChild(container);

Reki.instantiate(3, 5)
|> Js.Promise.(
     then_(reki => {
       let sum = reki->Reki.sum3(10) |> string_of_int;
       ReactDOMRe.render(<div> {React.string(sum)} </div>, container)
       |> resolve;
     })
   );
