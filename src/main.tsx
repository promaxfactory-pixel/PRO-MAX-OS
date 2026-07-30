import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import "./index.css";
import "./i18n";

const savedTheme = localStorage.getItem('promax-theme') || 'dark';
document.documentElement.setAttribute('data-theme', savedTheme);
const savedMode = localStorage.getItem('promax-work-mode') || 'professional';
document.documentElement.setAttribute('data-mode', savedMode);
const savedLang = localStorage.getItem('i18nextLng') || 'ar';
const rtlLangs = ['ar', 'ur'];
if (rtlLangs.includes(savedLang)) {
  document.documentElement.dir = 'rtl';
} else {
  document.documentElement.dir = 'ltr';
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>
);
