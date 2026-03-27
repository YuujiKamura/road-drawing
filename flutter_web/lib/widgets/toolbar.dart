import 'package:flutter/material.dart';

class Toolbar extends StatelessWidget {
  final VoidCallback onAddRow;
  final VoidCallback onDeleteRow;
  final VoidCallback onPreview;
  final VoidCallback onDownload;

  const Toolbar({
    super.key,
    required this.onAddRow,
    required this.onDeleteRow,
    required this.onPreview,
    required this.onDownload,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: Color(0xFF333333))),
      ),
      child: Wrap(
        spacing: 6,
        runSpacing: 4,
        alignment: WrapAlignment.end,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          const Text('Grid Editor',
              style: TextStyle(color: Color(0xFFAAAAAA), fontWeight: FontWeight.bold, fontSize: 13)),
          _button('+ Row', onAddRow),
          _button('- Row', onDeleteRow),
          _button('Preview', onPreview),
          _button('DXF', onDownload),
        ],
      ),
    );
  }

  Widget _button(String label, VoidCallback onPressed) {
    return OutlinedButton(
      onPressed: onPressed,
      style: OutlinedButton.styleFrom(
        foregroundColor: const Color(0xFFCCCCCC),
        side: const BorderSide(color: Color(0xFF555555)),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
        textStyle: const TextStyle(fontSize: 13),
      ),
      child: Text(label),
    );
  }
}
