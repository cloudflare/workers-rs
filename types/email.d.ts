/*
 * Email only types from @cloudflare/worker-types. Valid as of 28/04/2026.
 * This file builds src/email.rs as auto-generated bindings using ts-gen.
 *
 * NOTE: All hand edits to the @cloudflare/worker-types are marked with an "EDIT:" comment.
 */

/**
 * The **`ExtendableEvent`** interface extends the lifetime of the `install` and `activate` events dispatched on the global scope as part of the service worker lifecycle.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/ExtendableEvent)
 */
declare abstract class ExtendableEvent extends Event {
  // EDIT: added infallible
  /**
   * The **`ExtendableEvent.waitUntil()`** method tells the event dispatcher that work is ongoing.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/ExtendableEvent/waitUntil)
   * 
   * @throws {never}
   */
  waitUntil(promise: Promise<any>): void;
}
/**
 * The returned data after sending an email
 */
interface EmailSendResult {
  /**
   * The Email Message ID
   */
  messageId: string;
}
// EDIT: upstream splits `EmailMessage` into a global `interface
// EmailMessage` (the instance shape) and a `let _EmailMessage: { new(): EmailMessage }`
// inside `declare module "cloudflare:email"`, exported as `EmailMessage`.
// At runtime they are the same type — TS only splits them because module
// declarations cannot directly declare a class with both a constructor
// and an instance shape readable from the outer scope.
//
// In Rust + wasm-bindgen, a single `pub type EmailMessage` with a
// `#[wasm_bindgen(constructor)]` fn naturally covers both roles, so we
// collapse the upstream pattern into one `class EmailMessage` inside the
// module declaration. Everything that referenced upstream's global
// `EmailMessage` now imports from `cloudflare:email`.
declare module "cloudflare:email" {
  /**
   * An email message that can be sent from a Worker.
   */
  class EmailMessage {
    constructor(from: string, to: string, raw: string | ReadableStream);
    /**
     * Envelope From attribute of the email message.
     */
    readonly from: string;
    /**
     * Envelope To attribute of the email message.
     */
    readonly to: string;
  }
  export { EmailMessage };
}
import { EmailMessage } from "cloudflare:email";

/**
 * An email message that is sent to a consumer Worker and can be rejected/forwarded.
 */
interface ForwardableEmailMessage extends EmailMessage {
  /**
   * Stream of the email message content.
   */
  readonly raw: ReadableStream<Uint8Array>;
  /**
   * An [Headers object](https://developer.mozilla.org/en-US/docs/Web/API/Headers).
   */
  readonly headers: Headers;
  /**
   * Size of the email message content.
   */
  readonly rawSize: number;
  // EDIT: added infallible
  /**
   * Reject this email message by returning a permanent SMTP error back to the connecting client including the given reason.
   * @param reason The reject reason.
   * @returns void
   * 
   * @throws {never}
   */
  setReject(reason: string): void;
  /**
   * Forward this email message to a verified destination address of the account.
   * @param rcptTo Verified destination address.
   * @param headers A [Headers object](https://developer.mozilla.org/en-US/docs/Web/API/Headers).
   * @returns A promise that resolves when the email message is forwarded.
   */
  forward(rcptTo: string, headers?: Headers): Promise<EmailSendResult>;
  /**
   * Reply to the sender of this email message with a new EmailMessage object.
   * @param message The reply message.
   * @returns A promise that resolves when the email message is replied.
   */
  reply(message: EmailMessage): Promise<EmailSendResult>;
}
/** A file attachment for an email message */
type EmailAttachment =
  | {
      disposition: "inline";
      contentId: string;
      filename: string;
      type: string;
      content: string | ArrayBuffer | ArrayBufferView;
    }
  | {
      disposition: "attachment";
      contentId?: undefined;
      filename: string;
      type: string;
      content: string | ArrayBuffer | ArrayBufferView;
    };
/** An Email Address */
interface EmailAddress {
  name: string;
  email: string;
}
/**
 * A binding that allows a Worker to send email messages.
 */
interface SendEmail {
  send(message: EmailMessage): Promise<EmailSendResult>;
  send(builder: {
    from: string | EmailAddress;
    to: string | string[];
    subject: string;
    replyTo?: string | EmailAddress;
    cc?: string | string[];
    bcc?: string | string[];
    headers?: Record<string, string>;
    text?: string;
    html?: string;
    attachments?: EmailAttachment[];
  }): Promise<EmailSendResult>;
}
declare abstract class EmailEvent extends ExtendableEvent {
  readonly message: ForwardableEmailMessage;
}
// declare type EmailExportedHandler<Env = unknown, Props = unknown> = (
//   message: ForwardableEmailMessage,
//   env: Env,
//   ctx: ExecutionContext<Props>,
// ) => void | Promise<void>;
