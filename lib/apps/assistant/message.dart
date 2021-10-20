import 'dart:io';

import 'package:flutter/material.dart';
import 'package:image_save/image_save.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:open_file/open_file.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/widgets/audio_player.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/global.dart';

import 'package:esse/apps/file/models.dart' show FileType, FileTypeExtension, parseFileType;
import 'package:esse/apps/assistant/models.dart';

class AssistantMessage extends StatelessWidget {
  final String name;
  final Message message;
  final List<String> answers;

  const AssistantMessage({Key? key, required this.name, required this.message, required this.answers}): super(key: key);

  Widget _showText(context, color, isDesktop, content, isMe) {
    final width = MediaQuery.of(context).size.width * 0.6;
    // text
    return Container(
      constraints: BoxConstraints(minWidth: 50, maxWidth: isDesktop ? width - 300.0 : width),
      padding: const EdgeInsets.symmetric(vertical: 10.0, horizontal: 14.0),
      decoration: BoxDecoration(
        color: isMe ? Color(0xFF6174FF) : color.primaryVariant,
        borderRadius: BorderRadius.circular(15.0),
      ),
      child: Text(content,
        style: TextStyle(
          color: isMe ? Colors.white : Color(0xFF1C1939),
          fontSize: 14.0)));
  }

  Widget _showImage(context, lang, color, content) {
    // image
    bool imageExsit = true;
    var thumImage;
    final imagePath = Global.imagePath + content;
    final thumPath = Global.thumbPath + content;
    if (FileSystemEntity.typeSync(thumPath) ==
      FileSystemEntityType.notFound) {
      imageExsit = false;
      thumImage = AssetImage('assets/images/image_missing.png');
    } else {
      thumImage = FileImage(File(thumPath));
    }
    return GestureDetector(
      onTap: imageExsit
      ? () => showShadowDialog(
        context,
        Icons.image_rounded,
        lang.album,
        Column(children: [
            Image(image: FileImage(File(imagePath)), fit: BoxFit.cover),
            SizedBox(height: 15.0),
            if (Platform.isAndroid || Platform.isIOS)
            InkWell(
              onTap: () async {
                Map<Permission, PermissionStatus> statuses = await [
                  Permission.storage,
                ].request();

                if (statuses[Permission.storage] == PermissionStatus.granted) {

                  // Save to album.
                  final data = await File(imagePath).readAsBytes();
                  final bool? success = await ImageSave.saveImage(data, content, albumName: "ESSE");
                  print(success);

                  Navigator.pop(context);
                }
              },
              hoverColor: Colors.transparent,
              child: Container(
                width: 200.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                decoration: BoxDecoration(
                  border: Border.all(color: color.primary),
                  borderRadius: BorderRadius.circular(10.0)),
                child: Center(child: Text(lang.download,
                    style: TextStyle(fontSize: 14.0, color: color.primary))),
              )
            ),
      ]))
      : () => {},
      child: Container(
        width: imageExsit ? 120.0 : 60.0,
        child: Image(image: thumImage, fit: BoxFit.cover),
    ));
  }

  Widget _showFile(context, lang, color, content) {
    // file
    bool fileExsit = true;
    Widget fileImage;
    final filePath = Global.filePath + content;
    if (FileSystemEntity.typeSync(filePath) ==
      FileSystemEntityType.notFound) {
      fileExsit = false;
      fileImage = Image(image: AssetImage('assets/images/image_missing.png'), fit: BoxFit.cover);
    } else {
      final params = parseFileType(content).params();
      fileImage = Icon(params[0], color: params[1], size: 36.0);
    }
    return GestureDetector(
      onTap: fileExsit
      ? () => showShadowDialog(
        context,
        Icons.folder_rounded,
        lang.files,
        Column(children: [
            Text(content),
            SizedBox(height: 15.0),
            Container(
              height: 100.0,
              child: fileImage,
            ),
            SizedBox(height: 15.0),
            InkWell(
              onTap: () => OpenFile.open(filePath),
              hoverColor: Colors.transparent,
              child: Container(
                width: 200.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                decoration: BoxDecoration(
                  border: Border.all(color: color.primary),
                  borderRadius: BorderRadius.circular(10.0)),
                child: Center(child: Text(lang.open,
                    style: TextStyle(fontSize: 14.0, color: color.primary))),
              )
            ),
      ]))
      : () => {},
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 10.0, horizontal: 15.0),
        decoration: BoxDecoration(
          color: const Color(0x40ADB0BB),
          borderRadius: BorderRadius.circular(15.0),
        ),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
            Container(
              height: 36.0,
              child: fileImage,
            ),
            Container(
              padding: const EdgeInsets.only(left: 5.0),
              width: 120.0,
              child: Text(content,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: fileExsit
                ? TextStyle(
                  color: color.onPrimary,
                  fontSize: 14.0,
                )
                : TextStyle(
                  color: color.onPrimary.withOpacity(0.5),
                  decoration: TextDecoration.lineThrough,
                  fontSize: 14.0,
              )),
            ),
    ])));
  }

  Widget _showRecord(content) {
    final raws = Message.showRecordTime(content);
    // text
    return Container(
      width: 120.0,
      padding: const EdgeInsets.symmetric(vertical: 10.0, horizontal: 10.0),
      decoration: BoxDecoration(
        color: const Color(0x40ADB0BB),
        borderRadius: BorderRadius.circular(15.0),
      ),
      child: RecordPlayer(path: Global.recordPath + raws[1], time: raws[0]),
    );
  }

  Widget _showContact(context, lang, color, content) {
    // contact [name, gid, addr, avatar]
    final infos = Message.showContact(content);
    final gid = 'EH' + infos[1].toUpperCase();

    return GestureDetector(
        onTap: () => showShadowDialog(
            context,
            Icons.person_rounded,
            lang.contact,
            Column(children: [
              Avatar(width: 100.0, name: infos[0], avatarPath: infos[3]),
              const SizedBox(height: 10.0),
              Text(infos[0]),
              const SizedBox(height: 10.0),
              const Divider(height: 1.0, color: Color(0x40ADB0BB)),
              const SizedBox(height: 10.0),
              _infoListTooltip(Icons.person, color.primary, gidText(gid), gidPrint(gid)),
              _infoListTooltip(Icons.location_on, color.primary, addrText(infos[2]), addrPrint(infos[2])),
              Container(
                width: 300.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                child: Row(children: [
                  Icon(Icons.turned_in, size: 20.0, color: color.primary),
                  const SizedBox(width: 20.0),
                  Expanded(child: Text(lang.fromContactCard(name))),
                ]),
              ),
              const SizedBox(height: 20.0),
              InkWell(
                  onTap: () => Navigator.pop(context),
                  hoverColor: Colors.transparent,
                  child: Container(
                    width: 200.0,
                    padding: const EdgeInsets.symmetric(vertical: 10.0),
                    decoration: BoxDecoration(
                        border: Border.all(color: color.primary),
                        borderRadius: BorderRadius.circular(10.0)),
                    child: Center(
                        child: Text(lang.ok,
                            style: TextStyle(
                                fontSize: 14.0, color: color.primary))),
                  )),
            ])),
        child: Container(
            padding:
                const EdgeInsets.symmetric(vertical: 10.0, horizontal: 10.0),
            width: 200.0,
            decoration: BoxDecoration(
              color: const Color(0x40ADB0BB),
              borderRadius: BorderRadius.circular(15.0),
            ),
            child:
                Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Row(children: [
                Avatar(width: 40.0, name: infos[0], avatarPath: infos[3]),
                Container(
                    width: 135.0,
                    padding: const EdgeInsets.only(left: 10.0),
                    child: Column(children: [
                      Text(infos[0],
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                              color: color.onPrimary, fontSize: 16.0)),
                      SizedBox(height: 5.0),
                      Text(gidPrint(gid),
                          style: TextStyle(color: Colors.grey, fontSize: 12.0)),
                    ])),
              ]),
              SizedBox(height: 5.0),
              const Divider(height: 1.0, color: Color(0x40ADB0BB)),
              SizedBox(height: 3.0),
              Text(lang.contactCard,
                  style: TextStyle(color: Colors.grey, fontSize: 10.0)),
            ])));
  }

  Widget _infoListTooltip(icon, color, text, short) {
    return Container(
      width: 300.0,
      padding: const EdgeInsets.symmetric(vertical: 10.0),
      child: Row(
        children: [
          Icon(icon, size: 20.0, color: color),
          const SizedBox(width: 20.0),
          Expanded(
            child: Tooltip(
              message: text,
              child: Text(short),
            )
          )
        ]
      ),
    );
  }

  Widget _show(context, color, lang, isDesktop, type, content, isMe) {
    if (type == MessageType.String) {
      return _showText(context, color, isDesktop, content, isMe);
    } else if (type == MessageType.Image) {
      return _showImage(context, lang, color, content);
    } else if (type == MessageType.File) {
      return _showFile(context, lang, color, content);
    } else if (type == MessageType.Contact) {
      return _showContact(context, lang, color, content);
    } else if (type == MessageType.Record) {
      return  _showRecord(content);
    } else if (type == MessageType.Answer) {
      final index = int.parse(content);
      final value = this.answers.length > index ? this.answers[index] : content;
      return _showText(context, color, isDesktop, value, isMe);
    }
    return _showText(context, color, isDesktop, content, isMe);
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final qShow = _show(context, color, lang, isDesktop, message.qType, message.qContent, true);
    final aShow = _show(context, color, lang, isDesktop, message.aType, message.aContent, false);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      child: Column(children: [
          Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Expanded(
                child: Align(
                  alignment: Alignment.topRight,
                  child: qShow,
                ),
              ),
          ]),
          Container(
            padding: EdgeInsets.only(top: 10.0),
            child: Row(children: [
                Spacer(),
                Text(message.time.toString(),
                  style: TextStyle(
                    color: color.onPrimary.withOpacity(0.5),
                    fontSize: 12.0)),
                SizedBox(width: 4.0),
                Icon(
                  message.aContent.length == 0 ? Icons.hourglass_top : Icons.done,
                  size: 12.0,
                  color: color.primary
                ),
          ])),
          const SizedBox(height: 4.0),
          Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Expanded(
                child: Align(
                  alignment: Alignment.topLeft,
                  child: aShow,
                ),
              ),
          ]),
    ]));
  }
}
