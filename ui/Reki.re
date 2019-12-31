type reki;

let instantiate: (int, int) => Js.Promise.t(reki) = [%bs.raw {|
  function(a, b) {
    return wasm_bindgen('./dist/wasm/reki_bg.wasm').then(() =>
      wasm_bindgen.Reki.new(a, b));
  }
|}];

[@bs.send] external sum3 : (reki, int) => int = "sum3";
