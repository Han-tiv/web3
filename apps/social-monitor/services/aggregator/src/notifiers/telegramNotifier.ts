import axios from 'axios';
import { Logger } from 'winston';

interface TelegramNotifierOptions {
  botToken: string;
  chatId: string;
  logger: Logger;
  parseMode?: 'Markdown' | 'MarkdownV2' | 'HTML';
  disableNotification?: boolean;
  dryRun?: boolean;
}

export class TelegramNotifier {
  private readonly botToken: string;
  private readonly chatId: string;
  private readonly logger: Logger;
  private readonly parseMode: 'Markdown' | 'MarkdownV2' | 'HTML';
  private readonly disableNotification: boolean;
  private readonly dryRun: boolean;

  constructor(options: TelegramNotifierOptions) {
    this.botToken = options.botToken;
    this.chatId = options.chatId;
    this.logger = options.logger;
    this.parseMode = options.parseMode ?? 'HTML';
    this.disableNotification = options.disableNotification ?? false;
    this.dryRun = options.dryRun ?? false;
  }

  /**
   * 发送 Telegram 文本消息，必要时支持 dry-run，仅记录日志不调用 API。
   */
  async sendMessage(text: string): Promise<void> {
    if (this.dryRun) {
      this.logger.info('Telegram dry-run，未真正发送消息', { text });
      return;
    }

    const url = `https://api.telegram.org/bot${this.botToken}/sendMessage`;

    try {
      await axios.post(url, {
        chat_id: this.chatId,
        text,
        parse_mode: this.parseMode,
        disable_notification: this.disableNotification
      });
      this.logger.info('Telegram 消息发送成功');
    } catch (error) {
      this.logger.error('Telegram 消息发送失败', { error });
      throw error;
    }
  }
}
