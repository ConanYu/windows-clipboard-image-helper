import {Button, Checkbox, InputNumber, message, Select, Skeleton, Radio, Upload as AntdUpload, UploadProps} from "antd";
import {invoke} from "@tauri-apps/api";
import React, {useState} from "react";
import {open} from "@tauri-apps/api/dialog";
import {InboxOutlined} from "@ant-design/icons";

const {Option} = Select;
const {Dragger} = AntdUpload;

function Upload() {
  const [messageApi, contextHolder] = message.useMessage();
  const props: UploadProps = {
    showUploadList: false,
    openFileDialogOnClick: false,
  };
  const onClick = () => {
    open({multiple: true, filters: [{name: '图片', extensions: ['png', 'jpg', 'jpeg']}]})
      .then(async (file) => {
        if (file) {
          let imagePath = [];
          if (typeof file === 'string') {
            imagePath.push(file);
          } else {
            imagePath = file;
          }
          return await invoke('upload_image', {image_path: imagePath});
        }
        return 'canceled';
      })
      .then((value) => {
        if (value !== 'canceled') {
          return messageApi.open({
            type: 'success',
            content: '已触发后台上传',
          });
        }
      })
      .catch((s: string) => {
        return messageApi.open({
          type: 'error',
          content: s,
        });
      });
  };
  return (
    <>
      {contextHolder}
      <div onClick={onClick}>
        <Dragger {...props}>
          <p className="ant-upload-drag-icon"><InboxOutlined/></p>
          <p className="ant-upload-text">点击此区域即可上传图片</p>
          <p className="ant-upload-hint">只能上传jpg格式或png格式的图片，上传的图片仅在本地保存和分析，上传速度可能比较慢。</p>
        </Dragger>
      </div>
    </>
  )
}

export default function Settings() {
  const [ready, setReady] = useState(false);
  const [autoStart, setAutoStart] = useState<boolean>(false);
  const [closeWindowType, setCloseWindowType] = useState<'QUERY' | 'EXIT' | 'BACKGROUND'>('QUERY');
  const [databaseLimitType, setDatabaseLimitType] = useState<'MB' | 'NUM'>('MB');
  const [databaseLimit, setDatabaseLimit] = useState<number>(1024);
  const [databaseLimitMbValid, setDatabaseLimitMbValid] = useState<boolean>(true);
  const [messageApi, contextHolder] = message.useMessage();
  if (!ready) {
    invoke('get_settings', {}).then((value: any) => {
      setAutoStart(value.auto_start);
      setDatabaseLimitType(value.database_limit_type);
      setDatabaseLimit(value.database_limit);
      setCloseWindowType(value.close_window_type);
      setReady(true);
    });
    return <Skeleton/>;
  }
  const Header = (props: { text: string }) => {
    return <h3>{props.text}</h3>;
  };
  return (
    <div style={{marginLeft: 5}}>
      <div style={{marginTop: 15}}/>
      <Header text="设置"/>
      <Checkbox checked={autoStart} onChange={(e) => {
        setAutoStart(e.target.checked);
      }}>开机自启</Checkbox>
      <div style={{marginTop: 10}}/>
      <Radio.Group onChange={(e) => {
        setCloseWindowType(e.target.value);
      }} value={closeWindowType}>
        <Radio value="QUERY">询问</Radio>
        <Radio value="EXIT">退出</Radio>
        <Radio value="BACKGROUND">后台运行</Radio>
      </Radio.Group>
      <div style={{marginTop: 10}}/>
      <InputNumber addonBefore="数据库存储上限" style={{width: 333}} onChange={(e) => {
        const x = Number(e);
        if (isNaN(x) || x <= 0) {
          setDatabaseLimitMbValid(false);
        } else {
          setDatabaseLimitMbValid(true);
          setDatabaseLimit(e as number);
        }
      }} addonAfter={(
        <Select defaultValue={databaseLimitType} style={{width: 100}} onChange={(e) => {
          setDatabaseLimit(1024);
          setDatabaseLimitType(e);
        }}>
          <Option value="MB">MB</Option>
          <Option value="NUM">个</Option>
        </Select>
      )} defaultValue={databaseLimit} status={databaseLimitMbValid ? '' : 'error'}/>
      <div style={{marginTop: 15}}/>
      {contextHolder}
      <Button type="primary" onClick={() => {
        if (databaseLimitMbValid) {
          invoke('set_settings', {
            settings: {
              auto_start: autoStart,
              database_limit_type: databaseLimitType,
              database_limit: databaseLimit,
              close_window_type: closeWindowType,
            }
          }).then(() => {
            messageApi.open({
              type: 'success',
              content: '保存成功',
            }).then(() => {
            });
          }).catch((msg: string) => {
            messageApi.open({
              type: 'error',
              content: msg,
            }).then(() => {
            });
          });
        }
      }}>确认</Button>
      <Header text="上传图片"/>
      <Upload/>
    </div>
  );
}