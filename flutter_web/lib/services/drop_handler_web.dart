import 'dart:js_interop';
import 'package:web/web.dart' as web;

/// Register HTML5 drag-and-drop handlers on document.body.
void registerDropHandlers({
  required void Function() onDragOver,
  required void Function() onDragLeave,
  required void Function(String content) onDrop,
}) {
  final body = web.document.body;
  if (body == null) return;

  body.addEventListener(
    'dragover',
    ((web.Event e) {
      e.preventDefault();
      onDragOver();
    }).toJS,
  );
  body.addEventListener(
    'dragleave',
    ((web.Event e) {
      onDragLeave();
    }).toJS,
  );
  body.addEventListener(
    'drop',
    ((web.Event e) {
      e.preventDefault();
      final de = e as web.DragEvent;
      final files = de.dataTransfer?.files;
      if (files == null || files.length == 0) return;
      final file = files.item(0)!;
      final reader = web.FileReader();
      reader.onload = ((web.Event _) {
        final content = (reader.result as JSString).toDart;
        onDrop(content);
      }).toJS;
      reader.readAsText(file);
    }).toJS,
  );
}
