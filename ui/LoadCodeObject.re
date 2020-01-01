let readFileToBuffer:
  ReactEvent.Form.t => Js.Promise.t(Js_typed_array.ArrayBuffer.t) = [%bs.raw
  {| function(e) { return e.target.files[0].arrayBuffer(); } |}
];

[@react.component]
let make = (~coLoaded: Js_typed_array.ArrayBuffer.t => unit) => {
  let onFileChange = e =>
    e
    |> readFileToBuffer
    |> Js.Promise.(then_(f => f |> coLoaded |> resolve))
    |> ignore;
  <div> <input type_="file" onChange=onFileChange /> </div>;
};
