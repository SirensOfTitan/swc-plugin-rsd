import { html } from "react-strict-dom";
import { html as h } from "react-strict-dom";
function App() {
  return (
    <h.div style={styles.root}>
      <html.span style={[styles.foo, styles.bar]} />
    </h.div>
  );
}
