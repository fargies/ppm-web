// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>
import { ChildProcess, spawn } from "node:child_process";

/** @type {ChildProcess} */
let server;

/** @type {import("vitest/node").BrowserCommand} */
export async function startServer({ }) {
  return new Promise((resolve, reject) => {
    server = spawn("cargo", ["make", "serve:test"], {
      cwd: "../..",
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env, RUST_LOG: "trace" },
      detached: true
    });

    let output = "";
    const onData = (chunk) => {
      console.log(chunk.toString());
      output += chunk.toString();

      if (
        output.includes("starting daemon") &&
        output.includes("starting ppm-web")
      ) {
        cleanupListeners();
        resolve();
      }
    };

    const onError = (error) => {
      cleanupListeners();
      reject(error);
    };

    const onExit = (code) => {
      cleanupListeners();
      reject(new Error(`Server exited before becoming ready (code ${code})`));
    };

    let timeout = setTimeout(() => {
      cleanupListeners();
      reject(new Error(`Server timeout`));
    }, 30000);

    const cleanupListeners = () => {
      server?.stdout.off("data", onData);
      server?.off("error", onError);
      server?.off("exit", onExit);
      clearTimeout(timeout);
    };

    server.stdout.on("data", onData);
    server.on("error", onError);
    server.on("exit", onExit);
  });
}

/** @type {import("vitest/node").BrowserCommand} */
export async function stopServer() {
  if (!server || server.exitCode !== null) return;

  return new Promise((resolve) => {
    console.log("stopping server");

    // Avoid hanging indefinitely if the child ignores SIGTERM.
    const timeout = setTimeout(() => {
      if (server.exitCode === null) {
        console.log("sending KILL signal");
        process.kill(-server.pid, "SIGKILL");
        server.kill("SIGKILL");
      }
    }, 5000);

    server.once("exit", () => {
      console.log("server stopped");
      clearTimeout(timeout);
      resolve();
    });

    console.log("sending TERM signal");
    process.kill(-server.pid, "SIGTERM");
  });
}

export default { startServer, stopServer };
