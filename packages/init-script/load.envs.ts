import { config } from "dotenv";
import { resolve } from "path";

function getEnvName() {
  if (process.env.NODE_ENV === "production") {
    return ".env.production";
  }
  if (process.env.NODE_ENV === "test") {
    return ".env.test";
  }
}

// tmporary no fallback
// Load from more specific env file to generic ->
[getEnvName(), ".env.test"].forEach((envFile) => {
  if (!envFile) return;
  config({
    path: resolve(__dirname, envFile),
  });
});
