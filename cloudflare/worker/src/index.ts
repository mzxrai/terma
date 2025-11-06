import { Container, getContainer } from "@cloudflare/containers";

export class TermaContainer extends Container {
  defaultPort = 8080;
  sleepAfter = '10m';
}

export default {
  async fetch(request: Request, env: WorkerEnv): Promise<Response> {
    // Use a singleton container instance for all requests
    const container = getContainer(env.TERMA_CONTAINER, "terma-singleton");
    return container.fetch(request);
  },
};

interface WorkerEnv {
  TERMA_CONTAINER: unknown;
  DATABASE_URL?: string;
  HOST?: string;
}
