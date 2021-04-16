// ignore: avoid_web_libraries_in_flutter
import 'dart:html';
import 'package:web_socket_channel/html.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

abstract class MyWsChannelImpl {
  static Future<WebSocketChannel> connect(Uri uri) async {
    // ignore: close_sinks
    var ws = WebSocket(uri.toString());
    return HtmlWebSocketChannel(ws);
  }
}
