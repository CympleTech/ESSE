import 'package:web_socket_channel/web_socket_channel.dart';

import 'MyWsChannel_unimpl.dart'
    if (dart.library.io) 'MyWsChannel_io.dart'
    if (dart.library.html) 'MyWsChannel_web.dart';

abstract class MyWsChannel {
  static Future<WebSocketChannel> connect(Uri uri) async {
    return MyWsChannelImpl.connect(uri);
  }
}
