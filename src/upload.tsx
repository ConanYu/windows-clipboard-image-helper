import {open} from "@tauri-apps/api/dialog"
import {message, Upload as AntdUpload, UploadProps} from "antd";
import {InboxOutlined} from "@ant-design/icons";
import {invoke} from "@tauri-apps/api";

const {Dragger} = AntdUpload;

export default function Upload() {
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