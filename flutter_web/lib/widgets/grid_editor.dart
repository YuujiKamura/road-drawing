import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/station_data.dart';

class GridEditor extends StatefulWidget {
  final List<StationData> stations;
  final ValueChanged<List<StationData>> onChanged;

  const GridEditor({
    super.key,
    required this.stations,
    required this.onChanged,
  });

  @override
  State<GridEditor> createState() => _GridEditorState();
}

class _GridEditorState extends State<GridEditor> {
  late List<StationData> _stations;
  int? _selectedIndex;

  @override
  void initState() {
    super.initState();
    _stations = List.from(widget.stations);
  }

  @override
  void didUpdateWidget(GridEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.stations != widget.stations) {
      _stations = List.from(widget.stations);
    }
  }

  void _notifyChange() {
    widget.onChanged(List.from(_stations));
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: DataTable(
        showCheckboxColumn: false,
        columnSpacing: 12,
        horizontalMargin: 12,
        headingRowColor: WidgetStateProperty.all(const Color(0xFF2A2A3E)),
        columns: const [
          DataColumn(label: Text('測点名')),
          DataColumn(label: Text('単延長L'), numeric: true),
          DataColumn(label: Text('幅員W'), numeric: true),
          DataColumn(label: Text('幅員右'), numeric: true),
        ],
        rows: List.generate(_stations.length, (i) {
          final s = _stations[i];
          final selected = _selectedIndex == i;
          return DataRow(
            selected: selected,
            onSelectChanged: (_) => setState(() => _selectedIndex = i),
            cells: [
              _editableCell(s.name, (v) { s.name = v; _notifyChange(); }),
              _numericCell(s.x, (v) { s.x = v; _notifyChange(); }),
              _numericCell(s.wl, (v) { s.wl = v; _notifyChange(); }),
              _numericCell(s.wr, (v) { s.wr = v; _notifyChange(); }),
            ],
          );
        }),
      ),
      ),
    );
  }

  DataCell _editableCell(String value, ValueChanged<String> onChanged) {
    return DataCell(
      TextFormField(
        initialValue: value,
        style: const TextStyle(fontSize: 13),
        decoration: const InputDecoration(border: InputBorder.none, isDense: true),
        onFieldSubmitted: onChanged,
      ),
    );
  }

  DataCell _numericCell(double value, ValueChanged<double> onChanged) {
    return DataCell(
      TextFormField(
        initialValue: value.toString(),
        style: const TextStyle(fontSize: 13),
        decoration: const InputDecoration(border: InputBorder.none, isDense: true),
        keyboardType: const TextInputType.numberWithOptions(decimal: true),
        inputFormatters: [FilteringTextInputFormatter.allow(RegExp(r'[\d.]'))],
        onFieldSubmitted: (v) => onChanged(double.tryParse(v) ?? 0.0),
      ),
    );
  }
}
