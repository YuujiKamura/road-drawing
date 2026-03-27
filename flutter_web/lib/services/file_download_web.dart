import 'dart:js_interop';
import 'package:web/web.dart' as web;

/// Trigger a browser file download with the given content and filename.
void triggerDownload(String content, String filename) {
  final bytes = content.toJS;
  final blob = web.Blob(
    [bytes].toJS,
    web.BlobPropertyBag(type: 'application/dxf'),
  );
  final url = web.URL.createObjectURL(blob);
  final anchor = web.document.createElement('a') as web.HTMLAnchorElement
    ..href = url
    ..download = filename;
  anchor.click();
  web.URL.revokeObjectURL(url);
}
