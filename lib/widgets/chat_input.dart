import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/widgets/emoji.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/audio_recorder.dart';
import 'package:esse/widgets/show_contact.dart';
import 'package:esse/widgets/transfer.dart';
import 'package:esse/apps/primitives.dart';
import 'package:esse/provider.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';

class ChatInput extends StatefulWidget {
  final int sid;
  final bool online;
  final bool waiting;
  final Function callback;
  final bool hasTransfer;
  final String transferTo;
  final bool emojiWidth;
  ChatInput({Key? key,
      required this.sid,
      required this.online,
      required this.callback,
      this.hasTransfer = true,
      this.waiting = false,
      this.transferTo = '',
      this.emojiWidth = false
  }) : super(key: key);

  @override
  ChatInputState createState() => ChatInputState();
}

class ChatInputState extends State<ChatInput> {
  TextEditingController controller = TextEditingController();
  FocusNode focus = FocusNode();

  bool _emojiShow = false;
  bool _sendShow = false;
  bool _menuShow = false;
  bool _recordShow = false;
  String _recordName = '';

  @override
  void initState() {
    super.initState();
    focus.addListener(() {
        if (focus.hasFocus) {
          setState(() {
              _emojiShow = false;
              _menuShow = false;
              _recordShow = false;
          });
        }
    });
  }

  _restore() {
    setState(() {
        focus.requestFocus();
        _emojiShow = false;
        _sendShow = false;
        _menuShow = false;
        _recordShow = false;
    });
  }

  _generateRecordPath() {
    this._recordName = DateTime.now().millisecondsSinceEpoch.toString() + '.m4a';
  }

  _record(int time) async {
    final raw = BaseMessage.rawRecordName(time, _recordName);
    _restore();
    widget.callback(MessageType.Record, raw);
  }

  _contact(ColorScheme color, AppLocalizations lang) {
    showShadowDialog(
      context,
      Icons.person_rounded,
      lang.contact,
      ContactList(callback: _contactCallback, multiple: false, filters: [], online: false),
      0.0
    );
  }

  _contactCallback(int id) {
    _restore();
    widget.callback(MessageType.Contact, id.toString());
  }

  _file() async {
    _restore();
    final file = await pickFile();
    if (file != null) {
      widget.callback(MessageType.File, file);
    }
  }

  _image() async {
    _restore();
    final image = await pickImage();
    if (image != null) {
      widget.callback(MessageType.Image, image);
    }
  }

  _message() {
    final value = controller.text.trim();
    if (value.length < 1) {
      return;
    }
    controller.text = '';
    _restore();
    widget.callback(MessageType.String, value);
  }

  void _emoji(value) {
    controller.text += value;
    setState(() {
        _sendShow = true;
    });
  }

  _tokenCallback(String hash, String to, String amount, String name) {
    _restore();
    widget.callback(
      MessageType.Transfer,
      BaseMessage.mergeTransfer(hash, to, amount, name)
    );
  }

  void _transfer(ColorScheme color, AppLocalizations lang) {
    showShadowDialog(
      context,
      Icons.paid,
      lang.transfer,
      Transfer(callback: _tokenCallback, to: widget.transferTo),
      0.0
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    if (widget.online) {
      return Column(
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
            child: Row(
              children: [
                GestureDetector(
                  onTap: () {
                    if (_recordShow) {
                      _recordShow = false;
                      focus.requestFocus();
                    } else {
                      _generateRecordPath();
                      setState(() {
                          _menuShow = false;
                          _emojiShow = false;
                          _recordShow = true;
                          focus.unfocus();
                      });
                    }
                  },
                  child: Container(width: 20.0,
                    child: Icon(Icons.mic_rounded, color: color.primary)),
                ),
                const SizedBox(width: 10.0),
                Expanded(
                  child: Container(
                    height: 40,
                    decoration: BoxDecoration(
                      color: color.surface,
                      borderRadius: BorderRadius.circular(15.0),
                    ),
                    child: TextField(
                      style: TextStyle(fontSize: 14.0),
                      textInputAction: TextInputAction.send,
                      onChanged: (value) {
                        if (value.length == 0 && _sendShow) {
                          setState(() {
                              _sendShow = false;
                          });
                        } else {
                          if (!_sendShow) {
                            setState(() {
                                _sendShow = true;
                            });
                          }
                        }
                      },
                      onSubmitted: (_v) => _message(),
                      decoration: InputDecoration(
                        hintText: 'Aa',
                        border: InputBorder.none,
                        contentPadding: EdgeInsets.only(
                          left: 15.0, right: 15.0, bottom: 7.0),
                      ),
                      controller: controller,
                      focusNode: focus,
                    ),
                  ),
                ),
                const SizedBox(width: 10.0),
                GestureDetector(
                  onTap: () {
                    if (_emojiShow) {
                      focus.requestFocus();
                    } else {
                      setState(() {
                          _menuShow = false;
                          _recordShow = false;
                          _emojiShow = true;
                          focus.unfocus();
                      });
                    }
                  },
                  child: Container(width: 20.0,
                    child: Icon(_emojiShow
                      ? Icons.keyboard_rounded
                      : Icons.emoji_emotions_rounded,
                      color: color.primary)),
                ),
                const SizedBox(width: 10.0),
                _sendShow
                ? GestureDetector(
                  onTap: _message,
                  child: Container(
                    width: 50.0,
                    height: 30.0,
                    decoration: BoxDecoration(
                      color: Color(0xFF6174FF),
                      borderRadius: BorderRadius.circular(10.0),
                    ),
                    child: Center(
                      child: Icon(Icons.send,
                        color: Colors.white, size: 20.0))),
                )
                : GestureDetector(
                  onTap: () {
                    if (_menuShow) {
                      focus.requestFocus();
                    } else {
                      setState(() {
                          _emojiShow = false;
                          _recordShow = false;
                          _menuShow = true;
                          focus.unfocus();
                      });
                    }
                  },
                  child: Container(width: 20.0,
                    child: Icon(Icons.add_circle_rounded, color: color.primary)),
                ),
              ],
            ),
          ),
          if (_emojiShow) Emoji(action: _emoji, emojiWidth: widget.emojiWidth),
          if (_recordShow)
          Container(height: 100.0,
            child: AudioRecorder(
              path: Global.recordPath + _recordName, onStop: _record),
          ),
          if (_menuShow)
          Container(
            height: 100.0,
            child: Wrap(
              spacing: 20.0,
              runSpacing: 20.0,
              alignment: WrapAlignment.center,
              children: <Widget>[
                _ExtensionButton(
                  enable: true,
                  icon: Icons.image_rounded,
                  text: lang.album,
                  action: _image,
                  bgColor: color.surface,
                  iconColor: color.primary),
                _ExtensionButton(
                  enable: true,
                  icon: Icons.folder_rounded,
                  text: lang.file,
                  action: _file,
                  bgColor: color.surface,
                  iconColor: color.primary),
                _ExtensionButton(
                  enable: true,
                  icon: Icons.person_rounded,
                  text: lang.contact,
                  action: () => _contact(color, lang),
                  bgColor: color.surface,
                  iconColor: color.primary),
                if (widget.hasTransfer)
                _ExtensionButton(
                  enable: widget.transferTo.length > 2,
                  icon: Icons.paid_rounded,
                  text: lang.transfer,
                  action: () => _transfer(color, lang),
                  bgColor: color.surface,
                  iconColor: color.primary),
              ],
            ),
          )
        ]
      );
    } else {
      if (widget.waiting) {
        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
          margin: const EdgeInsets.all(10.0),
          decoration: BoxDecoration(
            color: Color(0x26ADB0BB),
            borderRadius: BorderRadius.circular(10.0)
          ),
          child: Center(child: Text(lang.connecting, style: TextStyle(color: color.primary))),
        );
      } else {
        return InkWell(
          onTap: () {
            context.read<AccountProvider>().updateActivedSession(widget.sid);
          },
          hoverColor: Colors.transparent,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 8.0),
            margin: const EdgeInsets.all(10.0),
            decoration: BoxDecoration(
              border: Border.all(color: color.primary),
              borderRadius: BorderRadius.circular(10.0)
            ),
            child: Center(child: Text(lang.reconnect, style: TextStyle(color: color.primary))),
          )
        );
      }
    }
  }
}

class _ExtensionButton extends StatelessWidget {
  final bool enable;
  final String text;
  final IconData icon;
  final VoidCallback action;
  final Color bgColor;
  final Color iconColor;

  const _ExtensionButton({
      Key? key,
      required this.icon,
      required this.text,
      required this.action,
      required this.bgColor,
      required this.iconColor,
      required this.enable,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: enable ? action : null,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            padding: const EdgeInsets.all(10.0),
            decoration: BoxDecoration(
              color: bgColor,
              borderRadius: BorderRadius.circular(15.0),
            ),
            child: Icon(icon, color: enable ? iconColor : Colors.grey, size: 36.0)),
          SizedBox(height: 5.0),
          Text(text, style: TextStyle(fontSize: 14.0)),
        ],
    ));
  }
}
