import { Injectable, NgZone } from '@angular/core';
import { Observable, Subscriber } from 'rxjs';

/**
* Server-Sent Events service
*/
@Injectable({
   providedIn: 'root'
})
export class EventService {
   private eventSource: EventSource;

   /**
    * constructor
    * @param zone - we need to use zone while working with server-sent events
    * because it's an asynchronous operations which are run outside of change detection scope
    * and we need to notify Angular about changes related to SSE events
    */
   constructor(private zone: NgZone) {
     const options = { withCredentials: true };

    this.eventSource = new EventSource('/events', options);    

   }


   /**
    * Method for establishing connection and subscribing to events from SSE
    * @param url - SSE server api path
    * @param options - configuration object for SSE
    * @param eventNames - all event names except error (listens by default) you want to listen to
    */
   connectToServerSentEvents(): Observable<MessageEvent> {
    let eventNames = ["ping", "state", "log"]
       return new Observable((subscriber: Subscriber<MessageEvent>) => {
           this.eventSource.onerror = error => {
               this.zone.run(() => subscriber.error(error));
           };

           eventNames.forEach((event: string) => {
               this.eventSource.addEventListener(event, data => {
                  this.zone.run(() => subscriber.next(data));
               });
           });
       });
   }


   /**
    * Method for closing the connection
    */
   close(): void {
       this.eventSource.close();
   }
}