import { defaultStyles as $_defaultStyles_1, resolveStyle as $_resolveStyle_2 } from "react-strict-dom/runtime";
import { html } from "react-strict-dom";
function App() {
    return <div {...$_resolveStyle_2($_defaultStyles_1.div)} data-element-src="input.js:4">
      <h1>Should not be transformed</h1>
      <strong {...$_resolveStyle_2($_defaultStyles_1.strong)} data-element-src="input.js:6">should be transformed</strong>
      <View style={{
        color: "red"
    }}/>
    </div>;
}
