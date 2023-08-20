import {Button, Checkbox, InputNumber, message, Select, Skeleton, Radio} from "antd";
import {invoke} from "@tauri-apps/api";
import React, {useState} from "react";

const {Option} = Select;

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
  return (
    <div style={{marginLeft: 5}}>
      <div style={{marginTop: 15}}/>
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
    </div>
  );
}