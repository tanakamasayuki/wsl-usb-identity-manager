import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";

// WebView2 offers its own menu - reload, print, save, inspect - none of which
// this application has any business offering. Rows that provide their own menu
// have already stopped the event before it reaches here. Text is still
// selectable and Ctrl+C still copies.
document.addEventListener("contextmenu", (event) => event.preventDefault());

export default mount(App, { target: document.getElementById("app")! });
