import { html as q } from "react-strict-dom";

function App() {
  return (
    <q.div role="none">
      <q.label for="for" />
      <q.input />
      <q.textarea />
      <q.input dir="rtl" />
      <q.textarea dir="rtl" />
      <q.button />
      <q.button type="submit" />
    </q.div>
  );
}
