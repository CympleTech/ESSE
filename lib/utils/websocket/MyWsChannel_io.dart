import 'dart:io';

import 'package:web_socket_channel/io.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

abstract class MyWsChannelImpl {
  static Future<WebSocketChannel> connect(Uri uri) async {
    try {
      final addrs = await InternetAddress.lookup(uri.host);
      if (addrs.isEmpty) {
        throw 'Unable to resolve host: ' + uri.host;
      }
      var hostAddr = (addrs..shuffle()).first.address;
      uri = Uri(
          fragment: uri.fragment,
          host: hostAddr,
          pathSegments: uri.pathSegments,
          port: uri.port,
          queryParameters: uri.queryParameters,
          scheme: uri.scheme,
          userInfo: uri.userInfo);
      // ignore: close_sinks
      final ws = await WebSocket.connect(uri.toString());
      return IOWebSocketChannel(ws);
    } catch (e) {
      return Future.error(e);
    }
  }
}
