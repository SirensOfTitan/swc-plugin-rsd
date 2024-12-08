import { defaultStyles as $_defaultStyles_1, resolveStyle as $_resolveStyle_2 } from "react-strict-dom/runtime";
import { html } from "react-strict-dom";
import { html as h } from "react-strict-dom";
function App() {
    return <div {...$_resolveStyle_2($_defaultStyles_1.div, styles.root)} data-element-src="input.js:4">
      <span {...$_resolveStyle_2($_defaultStyles_1.span, styles.foo, styles.bar)} data-element-src="input.js:5"/>
    </div>;
}
