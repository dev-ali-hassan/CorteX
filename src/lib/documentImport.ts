const TEXT_EXTENSIONS = new Set(["txt", "md", "markdown", "csv", "json", "log"]);

type CompressionFormatName = "deflate" | "deflate-raw";

type CompressionStreamConstructor = new (format: CompressionFormatName) => {
  writable: WritableStream<Uint8Array>;
  readable: ReadableStream<Uint8Array>;
};

export async function extractTextFromDocument(file: File) {
  const extension = getExtension(file.name);
  const bytes = new Uint8Array(await file.arrayBuffer());

  if (TEXT_EXTENSIONS.has(extension)) {
    return normalizeText(new TextDecoder("utf-8").decode(bytes));
  }

  if (extension === "rtf") {
    return normalizeText(stripRtf(new TextDecoder("utf-8").decode(bytes)));
  }

  if (extension === "docx") {
    return normalizeText(await extractDocxText(bytes));
  }

  if (extension === "pdf") {
    return normalizeText(await extractPdfText(bytes));
  }

  throw new Error("Use TXT, MD, RTF, DOCX, or PDF files.");
}

function getExtension(name: string) {
  return name.split(".").pop()?.toLowerCase() ?? "";
}

function normalizeText(value: string) {
  return value
    .replace(/\r/g, "\n")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n{3,}/g, "\n\n")
    .replace(/[ \t]{2,}/g, " ")
    .trim();
}

function stripRtf(value: string) {
  return value
    .replace(/\\par[d]?/g, "\n")
    .replace(/\\'[0-9a-fA-F]{2}/g, (match) => String.fromCharCode(Number.parseInt(match.slice(2), 16)))
    .replace(/\\[a-zA-Z]+-?\d* ?/g, "")
    .replace(/[{}]/g, " ");
}

async function extractDocxText(bytes: Uint8Array) {
  const files = await readZipEntries(bytes);
  const documentParts = Array.from(files.entries())
    .filter(([name]) => /^word\/(document|header\d+|footer\d+)\.xml$/i.test(name))
    .sort(([a], [b]) => Number(a.includes("document.xml")) - Number(b.includes("document.xml")));

  const textParts = documentParts.map(([, content]) => xmlToText(new TextDecoder("utf-8").decode(content)));
  const text = textParts.filter(Boolean).join("\n\n");
  if (!text.trim()) {
    throw new Error("No readable text found in this Word document.");
  }
  return text;
}

async function readZipEntries(bytes: Uint8Array) {
  const entries = new Map<string, Uint8Array>();
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const eocdOffset = findEndOfCentralDirectory(view);
  if (eocdOffset < 0) {
    throw new Error("This DOCX file could not be opened.");
  }

  const entryCount = view.getUint16(eocdOffset + 10, true);
  let centralOffset = view.getUint32(eocdOffset + 16, true);

  for (let index = 0; index < entryCount; index += 1) {
    if (view.getUint32(centralOffset, true) !== 0x02014b50) {
      break;
    }

    const compression = view.getUint16(centralOffset + 10, true);
    const compressedSize = view.getUint32(centralOffset + 20, true);
    const fileNameLength = view.getUint16(centralOffset + 28, true);
    const extraLength = view.getUint16(centralOffset + 30, true);
    const commentLength = view.getUint16(centralOffset + 32, true);
    const localOffset = view.getUint32(centralOffset + 42, true);
    const name = decodeUtf8(bytes.subarray(centralOffset + 46, centralOffset + 46 + fileNameLength));
    const localNameLength = view.getUint16(localOffset + 26, true);
    const localExtraLength = view.getUint16(localOffset + 28, true);
    const dataStart = localOffset + 30 + localNameLength + localExtraLength;
    const compressed = bytes.subarray(dataStart, dataStart + compressedSize);

    if (compression === 0) {
      entries.set(name, compressed);
    } else if (compression === 8) {
      entries.set(name, await inflate(compressed, "deflate-raw"));
    }

    centralOffset += 46 + fileNameLength + extraLength + commentLength;
  }

  return entries;
}

function findEndOfCentralDirectory(view: DataView) {
  const min = Math.max(0, view.byteLength - 66000);
  for (let offset = view.byteLength - 22; offset >= min; offset -= 1) {
    if (view.getUint32(offset, true) === 0x06054b50) {
      return offset;
    }
  }
  return -1;
}

function xmlToText(xml: string) {
  return decodeXmlEntities(
    xml
      .replace(/<w:tab\s*\/>/g, "\t")
      .replace(/<w:br\s*\/>|<\/w:p>/g, "\n")
      .replace(/<[^>]+>/g, " ")
  );
}

async function extractPdfText(bytes: Uint8Array) {
  const raw = decodeLatin1(bytes);
  const pieces = [
    extractPdfTextOperators(raw),
    ...(await Promise.all(extractPdfStreams(bytes, raw).map((stream) => stream.then(extractPdfTextOperators).catch(() => ""))))
  ];
  const text = pieces.join("\n").replace(/\\([()\\])/g, "$1");
  if (!text.trim()) {
    throw new Error("No readable text found in this PDF.");
  }
  return text;
}

function extractPdfStreams(bytes: Uint8Array, raw: string) {
  const streams: Array<Promise<string>> = [];
  const streamPattern = /stream\r?\n/g;
  let match: RegExpExecArray | null;
  while ((match = streamPattern.exec(raw))) {
    const start = match.index + match[0].length;
    const end = raw.indexOf("endstream", start);
    if (end < 0) {
      continue;
    }
    const streamBytes = bytes.subarray(start, end);
    streams.push(inflate(streamBytes, "deflate").then(decodeLatin1));
  }
  return streams;
}

function extractPdfTextOperators(value: string) {
  const text: string[] = [];
  const literalPattern = /\((?:\\.|[^\\)])*\)\s*T[jJ]/g;
  const arrayPattern = /\[((?:\s*\((?:\\.|[^\\)])*\)\s*-?\d*)+)\]\s*TJ/g;
  let match: RegExpExecArray | null;

  while ((match = literalPattern.exec(value))) {
    text.push(unescapePdfLiteral(match[0].slice(1, match[0].lastIndexOf(")"))));
  }

  while ((match = arrayPattern.exec(value))) {
    const inner = match[1];
    const values = inner.match(/\((?:\\.|[^\\)])*\)/g) ?? [];
    text.push(values.map((item) => unescapePdfLiteral(item.slice(1, -1))).join(""));
  }

  return text.join("\n");
}

function unescapePdfLiteral(value: string) {
  return value
    .replace(/\\n/g, "\n")
    .replace(/\\r/g, "\n")
    .replace(/\\t/g, "\t")
    .replace(/\\b/g, "")
    .replace(/\\f/g, "")
    .replace(/\\([()\\])/g, "$1")
    .replace(/\\([0-7]{1,3})/g, (_, octal: string) => String.fromCharCode(Number.parseInt(octal, 8)));
}

async function inflate(bytes: Uint8Array, format: CompressionFormatName) {
  const CompressionStreamRef = globalThis.CompressionStream as unknown as CompressionStreamConstructor | undefined;
  if (!CompressionStreamRef) {
    throw new Error("Compressed documents are not supported on this system.");
  }

  const payload = new Uint8Array(bytes).buffer;
  const stream = new Blob([payload]).stream().pipeThrough(new CompressionStreamRef(format));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

function decodeUtf8(bytes: Uint8Array) {
  return new TextDecoder("utf-8").decode(bytes);
}

function decodeLatin1(bytes: Uint8Array) {
  return new TextDecoder("latin1").decode(bytes);
}

function decodeXmlEntities(value: string) {
  return value
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, "\"")
    .replace(/&apos;/g, "'");
}
