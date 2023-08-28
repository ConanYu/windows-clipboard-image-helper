import {useState} from "react";
import {Button, Checkbox, FloatButton, Modal} from "antd";
import {AppstoreOutlined, ArrowLeftOutlined, SettingOutlined} from "@ant-design/icons";
import Index from "./index";
import Settings from "./settings";
import Detail from "./detail";
import {listen, TauriEvent} from "@tauri-apps/api/event";
import {invoke} from "@tauri-apps/api";

export default function App() {
  const [pageInfo, setPageInfo] = useState<'index' | 'settings' | 'detail'>('index');
  const [imageId, setImageId] = useState<number>(0);
  const [openConfirmCloseModal, setOpenConfirmCloseModal] = useState(false);
  const [confirmCloseChecked, setConfirmCloseChecked] = useState(false);
  const closeModal = () => {
    setConfirmCloseChecked(false);
    setOpenConfirmCloseModal(false);
  };
  const closeWindow = (force: boolean) => {
    invoke('close_window', {force, remember: confirmCloseChecked}).then(() => {
      closeModal();
    });
  };
  listen(TauriEvent.WINDOW_CLOSE_REQUESTED, () => {
    invoke('get_settings', {}).then((settings: any) => {
      const closeWindowType = settings?.close_window_type;
      if (closeWindowType === 'EXIT') {
        closeWindow(true);
      } else if (closeWindowType === 'BACKGROUND') {
        closeWindow(false);
      } else {
        setOpenConfirmCloseModal(true);
      }
    });
  }).then(() => {
  });
  const Main = (() => {
    if (pageInfo === 'index') {
      return (
        <>
          <Index jumpDetailPage={(id) => {
            setPageInfo('detail');
            setImageId(id);
          }}/>
          <FloatButton icon={<SettingOutlined/>} style={{right: 24}} type="default" onClick={() => {
            setPageInfo('settings');
          }}></FloatButton>
        </>
      );
    } else if (pageInfo === 'settings') {
      return (
        <>
          <Settings/>
          <FloatButton icon={<AppstoreOutlined/>} type="default" style={{right: 36}} onClick={() => {
            setPageInfo('index');
          }}></FloatButton>
        </>
      );
    } else if (pageInfo === 'detail') {
      return (
        <>
          <div>
            <Button size="large" icon={<ArrowLeftOutlined/>} type="link" onClick={() => {
              setPageInfo('index');
            }}>返回</Button>
          </div>
          <Detail imageId={imageId} jumpIndex={() => {
            setPageInfo('index');
          }}/>
        </>
      );
    }
    const _: never = pageInfo;
    return <></>;
  })();
  return (
    <>
      <Modal title="确认关闭" open={openConfirmCloseModal} footer={[
        <Button key="return" onClick={closeModal}>返回</Button>,
        <Button key="exit" danger onClick={() => closeWindow(true)}>退出</Button>,
        <Button key="backend" type="primary" ghost onClick={() => closeWindow(false)}>后台运行</Button>,
      ]} onCancel={closeModal}>
        <Checkbox checked={confirmCloseChecked} onChange={(e) => {
          setConfirmCloseChecked(e.target.checked);
        }}>下次不再提醒</Checkbox>
      </Modal>
      {Main}
    </>
  );
}