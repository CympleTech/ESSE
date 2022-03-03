import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/relative_time.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/chat_message.dart';
import 'package:esse/widgets/chat_input.dart';
import 'package:esse/options.dart';
import 'package:esse/account.dart' show Language, LanguageExtension;
import 'package:esse/rpc.dart';

import 'package:esse/apps/primitives.dart';

class Message extends BaseMessage {
  Message.fromList(List params) {
    this.id = params[0];
    this.isMe = params[1];
    this.type = MessageTypeExtension.fromInt(params[2]);
    this.content = params[3];
    this.time = RelativeTime.fromInt(params[4]);
    this.isDelivery = true;
  }
}

class JarvisDetail extends StatefulWidget {
  JarvisDetail({Key? key}) : super(key: key);

  @override
  _JarvisDetailState createState() => _JarvisDetailState();
}

class _JarvisDetailState extends State<JarvisDetail> {
  TextEditingController textController = TextEditingController();
  FocusNode textFocus = FocusNode();
  bool _emojiShow = false;
  bool _sendShow = false;
  bool _menuShow = false;
  bool _recordShow = false;
  String _recordName = '';

  Language _language = Language.English;
  List<String> _answers = [];
  Map<int, Message> _messages = {};

  @override
  initState() {
    super.initState();

    rpc.addListener('jarvis-create', _create);

    textFocus.addListener(() {
        if (textFocus.hasFocus) {
          setState(() {
              _emojiShow = false;
              _menuShow = false;
              _recordShow = false;
          });
        }
    });

    Future.delayed(Duration.zero, () async {
        this._language = LanguageExtension.fromLocale(context.read<Options>().locale);
        _load();
    });
  }

  _load() async {
    this._messages.clear();
    final res = await httpPost('jarvis-list', []);
    if (res.isOk) {
      _list(res.params);
    } else {
      print(res.error);
    }
  }

  /// list message with friend.
  _list(List params) {
    params.forEach((param) {
        this._messages[param[0]] = Message.fromList(param);
    });
    setState(() {});
  }

  /// friend send message to me.
  _create(List params) {
    final msg = Message.fromList(params);
    this._messages[msg.id] = msg;
    setState(() {});
  }

  _send(MessageType mtype, String raw) {
    rpc.send('jarvis-create', [this._language.toInt(), mtype.toInt(), raw]);
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final recentMessageKeys = this._messages.keys.toList().reversed.toList();

    return Scaffold(
      appBar: AppBar(
        automaticallyImplyLeading: false,
        leading: isDesktop ? null : IconButton(icon: Icon(Icons.arrow_back),
          onPressed: () => Navigator.pop(context)),
        title: Text(lang.jarvis,
          maxLines: 1, overflow: TextOverflow.ellipsis),
        bottom: isDesktop ? PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)): null,
      ),
      body: Column(
        children: [
          Expanded(
            child: ListView.builder(
              padding: EdgeInsets.symmetric(horizontal: 20.0),
              itemCount: recentMessageKeys.length,
              reverse: true,
              itemBuilder: (BuildContext context, index) => ChatMessage(
                fpid: '',
                name: lang.jarvis,
                message: this._messages[recentMessageKeys[index]]!,
              )
          )),
          ChatInput(sid: 0, online: true, callback: _send, hasTransfer: false),
        ]
      )
    );
  }
}
