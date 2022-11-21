(() => {
    const {opAsync, print } = globalThis.__bootstrap.core;
    const { Console } = globalThis.__bootstrap.console;
    const { URL } = globalThis.__bootstrap.url;
    const { ReadableStream } = globalThis.__bootstrap.streams;
    const { fetch, Headers, Request, Response } = globalThis.__bootstrap.fetch;
  
    globalThis.Flubber = {
      print(...msg) {
        opAsync("op_print", ...msg);
      }
    };
  

    function printer(msg, logLevel) {
      const useStderr = logLevel > 1; // 0 = debug, 1 = info, 2 = warn, 3 = error
      print(msg, useStderr);
    }
  
    globalThis.console = new Console(printer);
    globalThis.URL = URL;
    globalThis.ReadableStream = ReadableStream;
    globalThis.fetch = fetch;
    globalThis.Headers = Headers;
    globalThis.Request = Request;
    globalThis.Response = Response;
  
    delete globalThis.__bootstrap;
  })();