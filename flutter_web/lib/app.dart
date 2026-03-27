import 'package:flutter/material.dart';
import 'models/station_data.dart';
import 'models/preview_data.dart';
import 'services/dxf_service.dart';
import 'services/drop_handler.dart';
import 'services/file_download.dart';
import 'services/preview_fallback.dart';
import 'wasm_bridge.dart';
import 'widgets/grid_editor.dart';
import 'widgets/toolbar.dart';
import 'widgets/dxf_preview.dart';

class MainLayout extends StatefulWidget {
  const MainLayout({super.key});

  @override
  State<MainLayout> createState() => _MainLayoutState();
}

class _MainLayoutState extends State<MainLayout> {
  List<StationData> _stations = [
    StationData(name: 'No.0', x: 0, wl: 3.45, wr: 3.55),
    StationData(name: 'No.1', x: 20, wl: 3.50, wr: 3.50),
    StationData(name: 'No.2', x: 40, wl: 3.55, wr: 3.55),
  ];
  late PreviewData _preview;
  bool _isDragOver = false;

  @override
  void initState() {
    super.initState();
    _preview = _computePreview();
    registerDropHandlers(
      onDragOver: () {
        if (!_isDragOver) setState(() => _isDragOver = true);
      },
      onDragLeave: () {
        if (_isDragOver) setState(() => _isDragOver = false);
      },
      onDrop: (content) {
        setState(() => _isDragOver = false);
        _handleFileDrop(content);
      },
    );
  }

  PreviewData _computePreview() {
    if (WasmBridge.isInitialized) {
      return DxfService.getPreview(_stations);
    }
    return PreviewFallback.calculate(_stations);
  }

  void _handleFileDrop(String content) {
    List<StationData> stations;
    if (WasmBridge.isInitialized) {
      final json = WasmBridge.parseCsv(content);
      if (json.contains('"error"')) return;
      stations = StationData.fromJsonList(json);
    } else {
      stations = StationData.fromCsv(content);
    }
    if (stations.isEmpty) return;
    setState(() {
      _stations = stations;
      _preview = _computePreview();
    });
  }

  void _addRow() {
    setState(() {
      final lastX = _stations.isNotEmpty ? _stations.last.x : 0.0;
      _stations = List.from(_stations)
        ..add(StationData(
          name: 'No.${_stations.length}',
          x: lastX + 20,
          wl: 3.0,
          wr: 3.0,
        ));
      _preview = _computePreview();
    });
  }

  void _deleteRow() {
    if (_stations.isNotEmpty) {
      setState(() {
        _stations = List.from(_stations)..removeLast();
        _preview = _computePreview();
      });
    }
  }

  void _onGridChanged(List<StationData> updated) {
    setState(() {
      _stations = updated;
      _preview = _computePreview();
    });
  }

  void _onPreview() {
    setState(() => _preview = _computePreview());
  }

  void _onDownload() {
    final dxf = DxfService.generateDxf(_stations);
    if (dxf.startsWith('ERROR:')) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(dxf)),
      );
      return;
    }
    triggerDownload(dxf, 'road_section.dxf');
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          Row(
            children: [
              SizedBox(
                width: 380,
                child: Container(
                  color: const Color(0xFF1E1E2E),
                  child: Column(
                    children: [
                      Toolbar(
                        onAddRow: _addRow,
                        onDeleteRow: _deleteRow,
                        onPreview: _onPreview,
                        onDownload: _onDownload,
                      ),
                      Expanded(
                        child: GridEditor(
                          stations: _stations,
                          onChanged: _onGridChanged,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const VerticalDivider(width: 1, color: Color(0xFF333333)),
              Expanded(
                child: DxfPreview(data: _preview),
              ),
            ],
          ),
          if (_isDragOver)
            Container(
              color: Colors.blue.withAlpha(51),
              child: const Center(
                child: Text(
                  'CSVファイルをドロップ',
                  style: TextStyle(fontSize: 24, color: Colors.white),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
