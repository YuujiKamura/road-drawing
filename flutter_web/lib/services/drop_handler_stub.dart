/// Stub for non-web platforms. Drop handlers do nothing.
void registerDropHandlers({
  required void Function() onDragOver,
  required void Function() onDragLeave,
  required void Function(String content) onDrop,
}) {
  // No-op on non-web platforms
}
