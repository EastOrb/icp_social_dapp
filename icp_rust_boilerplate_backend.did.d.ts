import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type Error = { 'AlreadyVoted' : { 'msg' : string } } |
  { 'NotFound' : { 'msg' : string } } |
  { 'UserNotFound' : { 'msg' : string } };
export interface Message {
  'id' : bigint,
  'upvotes' : bigint,
  'title' : string,
  'updated_at' : [] | [bigint],
  'downvoted_users' : Array<string>,
  'body' : string,
  'created_at' : bigint,
  'upvoted_users' : Array<string>,
  'downvotes' : bigint,
  'attachment_url' : string,
}
export interface MessagePayload {
  'title' : string,
  'body' : string,
  'attachment_url' : string,
}
export type Result = { 'Ok' : Message } |
  { 'Err' : Error };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : Error };
export interface _SERVICE {
  'add_message' : ActorMethod<[MessagePayload], [] | [Message]>,
  'delete_message' : ActorMethod<[bigint], Result>,
  'downvote_message' : ActorMethod<[bigint, string], Result_1>,
  'get_message' : ActorMethod<[bigint], Result>,
  'reward_upvote' : ActorMethod<[string], Result_1>,
  'update_message' : ActorMethod<[bigint, MessagePayload], Result>,
  'upvote_message' : ActorMethod<[bigint, string], Result_1>,
}
