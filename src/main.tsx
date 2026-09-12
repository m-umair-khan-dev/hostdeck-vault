import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import * as Sentry from "@sentry/react";

Sentry.init({
  dsn: "https://93c09cda03ee28115f0fcd5462523775@o4511788661669888.ingest.de.sentry.io/4512074160799824",
  integrations: [
    Sentry.browserTracingIntegration(),
  ],
  tracesSampleRate: 1.0,
});
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
