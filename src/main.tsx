import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import PopupApp from "./PopupApp";
import "./styles.css";

const params = new URLSearchParams(window.location.search);
const isPopup = params.get("view") === "popup";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{isPopup ? <PopupApp /> : <App />}</React.StrictMode>
);
