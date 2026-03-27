import '../models/station_data.dart';
import '../models/preview_data.dart';
import '../wasm_bridge.dart';

class DxfService {
  /// Get preview geometry from current stations
  static PreviewData getPreview(List<StationData> stations) {
    final csv = StationData.toCsv(stations);
    final json = WasmBridge.getPreviewData(csv);
    return PreviewData.fromJson(json);
  }

  /// Generate DXF string for download
  static String generateDxf(List<StationData> stations) {
    final csv = StationData.toCsv(stations);
    return WasmBridge.generateDxf(csv);
  }
}
