import { html } from "react-strict-dom";

function App() {
  return (
    <html.div>
      <h1>Should not be transformed</h1>
      <html.strong>should be transformed</html.strong>
      <View style={{ color: "red" }} />
    </html.div>
  );
}
