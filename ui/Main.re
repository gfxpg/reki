[@react.component]
let make = () => {
  let (coBuffer, setCoBuffer) = React.useState(() => None);

  let onCoLoad = buf => setCoBuffer(_ => Some(buf))->ignore;

  switch (coBuffer) {
  | Some(buf) => <h1>{React.string("Code object loaded")}</h1>
  | _ => <LoadCodeObject coLoaded=onCoLoad />
  }
};
