import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/emoji.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/audio_recorder.dart';
import 'package:esse/widgets/show_contact.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';

import 'package:esse/apps/assistant/models.dart';
import 'package:esse/apps/assistant/provider.dart';
import 'package:esse/apps/assistant/message.dart';
import 'package:esse/apps/assistant/answer.dart';

class AssistantDetail extends StatefulWidget {
  const AssistantDetail({Key? key}) : super(key: key);

  @override
  _AssistantDetailState createState() => _AssistantDetailState();
}

class _AssistantDetailState extends State<AssistantDetail> {
  TextEditingController textController = TextEditingController();
  FocusNode textFocus = FocusNode();
  bool emojiShow = false;
  bool sendShow = false;
  bool menuShow = false;
  bool recordShow = false;
  String _recordName = '';
  List<String> answers = [];

  @override
  initState() {
    super.initState();
    Future.delayed(Duration.zero, () async {
        Provider.of<AssistantProvider>(context, listen: false).actived();
        final options = context.read<Options>();
        this.answers = await loadAnswers(options.locale);
        setState(() {});
    });
    textFocus.addListener(() {
        if (textFocus.hasFocus) {
          setState(() {
              emojiShow = false;
              menuShow = false;
              recordShow = false;
          });
        }
    });
  }

  @override
  void deactivate() {
    Provider.of<AssistantProvider>(context, listen: false).inactived();
    super.deactivate();
  }

  _generateRecordPath() {
    this._recordName = DateTime.now().millisecondsSinceEpoch.toString() + '_assistant.m4a';
  }

  void _sendMessage() async {
    if (textController.text.length < 1) {
      return;
    }

    final value = textController.text.trim();
    final aType = (value.endsWith('?') || value.endsWith('？')) ? MessageType.Answer : MessageType.String;
    context.read<AssistantProvider>().create(aType, textController.text);

    setState(() {
        textController.text = '';
        textFocus.requestFocus();

        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _selectEmoji(value) {
    textController.text += value;
  }

  void _sendImage() async {
    final image = await pickImage();
    if (image != null) {
      context.read<AssistantProvider>().create(MessageType.Image, image);
    }
    setState(() {
        textFocus.requestFocus();
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendFile() async {
    final file = await pickFile();
    if (file != null) {
      context.read<AssistantProvider>().create(MessageType.File, file);
    }
    setState(() {
        textFocus.requestFocus();
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendRecord(int time) async {
    final raw = Message.rawRecordName(time, _recordName);
    context.read<AssistantProvider>().create(MessageType.Record, raw);

    setState(() {
        textFocus.requestFocus();
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  _callback(int id) {
    context.read<AssistantProvider>().create(MessageType.Contact, id.toString());
    setState(() {
        textFocus.requestFocus();
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendContact(ColorScheme color, AppLocalizations lang) {
    showShadowDialog(
      context,
      Icons.person_rounded,
      lang.contact,
      ContactList(callback: _callback, multiple: false)
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final recentMessages = context.watch<AssistantProvider>().messages;
    final recentMessageKeys = recentMessages.keys.toList().reversed.toList();

    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Container(
              padding: EdgeInsets.only(left: 20.0, right: 20.0, top: 10.0, bottom: 6.0),
              child: Row(
                children: [
                  if (!isDesktop)
                  GestureDetector(
                    onTap: () {
                      Navigator.pop(context);
                    },
                    child: Container(
                      width: 20.0,
                      child:
                      Icon(Icons.arrow_back, color: color.primary)),
                  ),
                  SizedBox(width: 15.0),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Jarvis',
                          style: TextStyle(fontWeight: FontWeight.bold),
                        ),
                        SizedBox(height: 5.0),
                        Text(lang.onlineActive,
                          style: TextStyle(color: color.onPrimary.withOpacity(0.5), fontSize: 12.0))
                      ],
                    ),
                  ),
                  SizedBox(width: 20.0),
                  GestureDetector(
                    onTap: () {},
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.phone_rounded,
                        color: Color(0x26ADB0BB))),
                  ),
                  SizedBox(width: 20.0),
                  GestureDetector(
                    onTap: () {},
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.videocam_rounded,
                        color: Color(0x26ADB0BB))),
                  ),
                  SizedBox(width: 20.0),
                  // PopupMenuButton<int>(
                  //   shape: RoundedRectangleBorder(
                  //     borderRadius: BorderRadius.circular(15)
                  //   ),
                  //   color: const Color(0xFFEDEDED),
                  //   child: Icon(Icons.more_vert_rounded, color: color.primary),
                  //   onSelected: (int value) {
                  //     if (value == 1) {
                  //       // TODO set top
                  //     }
                  //   },
                  //   itemBuilder: (context) {
                  //     return <PopupMenuEntry<int>>[
                  //       _menuItem(Color(0xFF6174FF), 1, Icons.vertical_align_top_rounded, lang.cancelTop),
                  //     ];
                  //   },
                  // )
                ]
              ),
            ),
            const Divider(height: 1.0, color: Color(0x40ADB0BB)),
            Expanded(
              child: ListView.builder(
                padding: EdgeInsets.symmetric(horizontal: 20.0),
                itemCount: recentMessageKeys.length,
                reverse: true,
                itemBuilder: (BuildContext context, index) => AssistantMessage(
                  name: 'Jarvis',
                  message: recentMessages[recentMessageKeys[index]]!,
                  answers: this.answers,
                )
            )),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
              child: Row(
                children: [
                  GestureDetector(
                    onTap: () async {
                      if (recordShow) {
                        recordShow = false;
                        textFocus.requestFocus();
                      } else {
                        _generateRecordPath();
                        setState(() {
                            menuShow = false;
                            emojiShow = false;
                            recordShow = true;
                            textFocus.unfocus();
                        });
                      }
                    },
                    child: Container(width: 20.0,
                      child: Icon(Icons.mic_rounded, color: color.primary)),
                  ),
                  SizedBox(width: 10.0),
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
                          if (value.length == 0 && sendShow) {
                            setState(() {
                                sendShow = false;
                            });
                          } else {
                            if (!sendShow) {
                              setState(() {
                                  sendShow = true;
                              });
                            }
                          }
                        },
                        onSubmitted: (_v) => _sendMessage(),
                        decoration: InputDecoration(
                          hintText: 'Aa',
                          border: InputBorder.none,
                          contentPadding: EdgeInsets.only(
                            left: 15.0, right: 15.0, bottom: 7.0),
                        ),
                        controller: textController,
                        focusNode: textFocus,
                      ),
                    ),
                  ),
                  SizedBox(width: 10.0),
                  GestureDetector(
                    onTap: () {
                      if (emojiShow) {
                        textFocus.requestFocus();
                      } else {
                        setState(() {
                            menuShow = false;
                            recordShow = false;
                            emojiShow = true;
                            textFocus.unfocus();
                        });
                      }
                    },
                    child: Container(
                      width: 20.0,
                      child: Icon(
                        emojiShow
                        ? Icons.keyboard_rounded
                        : Icons.emoji_emotions_rounded,
                        color: color.primary)),
                  ),
                  SizedBox(width: 10.0),
                  sendShow
                  ? GestureDetector(
                    onTap: _sendMessage,
                    child: Container(
                      width: 50.0,
                      height: 30.0,
                      decoration: BoxDecoration(
                        color: Color(0xFF6174FF),
                        borderRadius: BorderRadius.circular(10.0),
                      ),
                      child: Center(
                        child: Icon(Icons.send, color: Colors.white, size: 20.0))),
                  )
                  : GestureDetector(
                    onTap: () {
                      if (menuShow) {
                        textFocus.requestFocus();
                      } else {
                        setState(() {
                            emojiShow = false;
                            recordShow = false;
                            menuShow = true;
                            textFocus.unfocus();
                        });
                      }
                    },
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.add_circle_rounded, color: color.primary)),
                  ),
                ],
              ),
            ),
            if (emojiShow) Emoji(action: _selectEmoji),
            if (recordShow)
            Container(
              height: 100.0,
              child: AudioRecorder(
                path: Global.recordPath + _recordName, onStop: _sendRecord),
            ),
            if (menuShow)
            Container(
              height: 100.0,
              child: Wrap(
                spacing: 20.0,
                runSpacing: 20.0,
                alignment: WrapAlignment.center,
                children: <Widget>[
                  ExtensionButton(
                    icon: Icons.image_rounded,
                    text: lang.album,
                    action: _sendImage,
                    bgColor: color.surface,
                    iconColor: color.primary),
                  ExtensionButton(
                    icon: Icons.folder_rounded,
                    text: lang.file,
                    action: _sendFile,
                    bgColor: color.surface,
                    iconColor: color.primary),
                  ExtensionButton(
                    icon: Icons.person_rounded,
                    text: lang.contact,
                    action: () => _sendContact(color, lang),
                    bgColor: color.surface,
                    iconColor: color.primary),
                ],
              ),
            )
          ],
        )
      )
    );
  }
}

class ExtensionButton extends StatelessWidget {
  final String text;
  final IconData icon;
  final VoidCallback action;
  final Color bgColor;
  final Color iconColor;

  const ExtensionButton({
      Key? key,
      required this.icon,
      required this.text,
      required this.action,
      required this.bgColor,
      required this.iconColor,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: action,
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
            child: Icon(icon, color: iconColor, size: 36.0)),
          SizedBox(height: 5.0),
          Text(text, style: TextStyle(fontSize: 14.0)),
        ],
    ));
  }
}
